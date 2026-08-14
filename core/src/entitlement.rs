//! SQL-3340: Atlas SQL Direct Cluster entitlement verification.
//!
//! Atlas SQL "Direct Cluster" is a paid, per-cluster toggle. When it is enabled, the Atlas
//! control plane writes a signed entitlement marker — an Ed25519 (EdDSA) compact JWS — into the
//! reserved `__sql_interface.__sql_status` collection on the user cluster. Drivers verify that
//! marker at connection time and refuse to connect when the interface is not entitled.
//!
//! The signing side of this contract lives in the `sql-interface` CNCP service
//! (10gen/mms PR #178521, SQL-3325); the token shape here must stay in lock-step with it.
//!
//! Scope notes:
//! * This gate applies **only to Atlas dedicated clusters**. On-prem / self-managed Enterprise
//!   deployments have no marker and must still be allowed to connect, so we detect an Atlas
//!   dedicated host from `hello.me` and skip the check otherwise.
//! * Fetching the JWKS public key is a separate ticket (task 28). It is stubbed here behind
//!   [`Ed25519KeyProvider`]; [`StubKeyProvider`] is wired in for now and unit tests inject a
//!   known key to exercise the full verification path.

use crate::util::run_command_with_retry;
use crate::{err::Result, Error};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use mongodb::{bson::doc, Client};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Issuer (`iss`) for normal markers signed by the CNCP SQL Interface service.
pub const ISSUER_NORMAL: &str = "mongosql-service";
/// Issuer (`iss`) for break-glass markers signed by the restricted support tool (SQL-3337).
pub const ISSUER_EMERGENCY: &str = "mongosql-emergency";

/// JWS algorithm the marker is signed with. We pin this on the verify side and never trust the
/// token's self-declared `alg` for anything else (classic JWT alg-confusion mitigation).
const EXPECTED_ALG: &str = "EdDSA";

const MARKER_DB: &str = "__sql_interface";
const MARKER_COLLECTION: &str = "__sql_status";
const MARKER_ID: &str = "entitlement";

// Atlas dedicated cluster hostnames always embed the cluster name before the first `-shard-`
// segment and resolve under `.mongodb.net`, e.g.
//   cluster0-shard-00-00.abc123.mongodb.net:27017        (replica set node)
//   cluster0-shard-00-mongos-g0.abc123.mongodb.net:27017 (mongos)
// The cluster name is the prefix before `-shard-`. On-prem / self-managed hosts do not match,
// which is exactly how we scope the entitlement gate to Atlas dedicated clusters only.
static ATLAS_DEDICATED_HOST: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^([^.]+)-shard-\d+.*\.mongodb\.net(:\d+)?$").unwrap());

/// Supplies the Ed25519 public key used to verify a marker signature, selected by the JWT `kid`.
///
/// The real implementation (task 28) fetches and caches keys from the JWKS well-known endpoint,
/// with a local-file fallback for air-gapped environments. It is abstracted so that verification
/// logic can be unit-tested with an injected key.
pub trait Ed25519KeyProvider {
    fn verifying_key(&self, kid: &str) -> Result<VerifyingKey>;
}

/// Placeholder key provider used until task 28 (ODBC public-key retrieval) lands.
///
/// It always fails, so any real Atlas dedicated cluster with the feature enabled will currently be
/// rejected with a clear message. That is acceptable while the feature is pre-GA and gated; the
/// happy path is exercised in tests via an injected [`Ed25519KeyProvider`].
pub struct StubKeyProvider;

impl Ed25519KeyProvider for StubKeyProvider {
    fn verifying_key(&self, _kid: &str) -> Result<VerifyingKey> {
        Err(Error::SqlInterfaceNotEntitled(
            "unable to verify the Atlas SQL interface entitlement: public key retrieval is not \
             yet implemented in this driver build (SQL-3340 depends on task 28)"
                .to_string(),
        ))
    }
}

/// Default JWKS well-known endpoint publishing the marker signing public keys (RFC 7517).
pub const DEFAULT_JWKS_URL: &str = "https://cloud.mongodb.com/.well-known/mongosql/jwks.json";

/// How long a fetched JWKS is considered fresh before a re-fetch (marker design: 24h).
const JWKS_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

// ---------------------------------------------------------------------------------------------
// Task 28 (SQL "ODBC driver — public key retrieval"): JWKS-backed key provider.
//
// This is the real [`Ed25519KeyProvider`] that replaces [`StubKeyProvider`]. It is scaffolded
// here so the verification path (SQL-3340) and key retrieval (task 28) dovetail without further
// changes to the trait or to `verify_token`. The pure JWKS parsing (`parse_jwks`) is fully
// implemented and unit-tested; the HTTP fetch, local-file fallback, and cache/refresh policy are
// implemented but should be reviewed and tuned by the task-28 owner (see TODOs below).
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

/// A single JSON Web Key. We only consume Ed25519 OKP signing keys; other entries are ignored.
#[derive(Debug, Deserialize)]
struct Jwk {
    kty: String,
    #[serde(default)]
    crv: Option<String>,
    kid: String,
    /// base64url-encoded 32-byte Ed25519 public key.
    x: String,
}

/// Parses a JWKS document body into a `kid -> VerifyingKey` map, keeping only Ed25519 OKP keys.
/// Pure and network-free so it can be unit-tested directly.
fn parse_jwks(body: &str) -> Result<HashMap<String, VerifyingKey>> {
    let jwks: Jwks = serde_json::from_str(body)
        .map_err(|e| not_entitled(format!("the SQL interface JWKS document is not valid: {e}")))?;

    let mut keys = HashMap::new();
    for jwk in jwks.keys {
        // Only Ed25519 OKP signing keys are relevant to the marker.
        if jwk.kty != "OKP" || jwk.crv.as_deref() != Some("Ed25519") {
            continue;
        }
        let raw = URL_SAFE_NO_PAD.decode(&jwk.x).map_err(|_| {
            not_entitled(format!(
                "SQL interface JWKS key '{}' has invalid base64url",
                jwk.kid
            ))
        })?;
        let bytes: [u8; 32] = raw.as_slice().try_into().map_err(|_| {
            not_entitled(format!(
                "SQL interface JWKS key '{}' is not a 32-byte key",
                jwk.kid
            ))
        })?;
        let key = VerifyingKey::from_bytes(&bytes).map_err(|_| {
            not_entitled(format!(
                "SQL interface JWKS key '{}' is not a valid Ed25519 key",
                jwk.kid
            ))
        })?;
        keys.insert(jwk.kid, key);
    }
    Ok(keys)
}

#[derive(Default)]
struct JwksCache {
    keys: HashMap<String, VerifyingKey>,
    fetched_at: Option<Instant>,
}

/// Fetches and caches Ed25519 public keys from the JWKS endpoint, with a local-file fallback for
/// air-gapped environments. Implements the caching behaviour from the marker design: 24h freshness,
/// re-fetch on cache miss (mid-rotation), and fall back to stale cached keys on fetch failure.
pub struct JwksKeyProvider {
    jwks_url: String,
    /// Air-gapped fallback: a local `jwks.json` path (from the `SQLPublicKeyPath` DSN option). When
    /// set, it is the source of truth and no HTTP request is made.
    local_jwks_path: Option<PathBuf>,
    http: reqwest::blocking::Client,
    // TODO(task-28): the marker design implies the 24h JWKS cache should be process-wide (shared
    // across connections), while the air-gapped file is per-connection. This holds a per-instance
    // cache; if the provider ends up constructed per-connection, promote this to a shared
    // singleton (e.g. a `LazyLock<Mutex<..>>` keyed by URL) so connections share fetched keys.
    cache: Mutex<JwksCache>,
    ttl: Duration,
}

impl JwksKeyProvider {
    pub fn new(jwks_url: impl Into<String>, local_jwks_path: Option<PathBuf>) -> Self {
        Self {
            jwks_url: jwks_url.into(),
            local_jwks_path,
            http: reqwest::blocking::Client::new(),
            cache: Mutex::new(JwksCache::default()),
            ttl: JWKS_CACHE_TTL,
        }
    }

    /// Constructs a provider against the default cloud JWKS endpoint, with an optional air-gapped
    /// local-file fallback (from the `SQLPublicKeyPath` DSN/connection-string option).
    pub fn with_defaults(local_jwks_path: Option<PathBuf>) -> Self {
        Self::new(DEFAULT_JWKS_URL, local_jwks_path)
    }

    /// Loads the raw JWKS document: from the local file when configured (air-gapped), else over
    /// HTTPS from the well-known endpoint.
    fn load_jwks_body(&self) -> Result<String> {
        if let Some(path) = &self.local_jwks_path {
            return std::fs::read_to_string(path).map_err(|e| {
                not_entitled(format!(
                    "unable to read the local SQL interface public key file '{}': {e}",
                    path.display()
                ))
            });
        }
        let resp = self.http.get(&self.jwks_url).send().map_err(|e| {
            not_entitled(format!(
                "unable to fetch the SQL interface JWKS from the registry: {e}"
            ))
        })?;
        // `send` only fails on transport errors; a 404/500 is a "successful" response whose body
        // would otherwise surface as a misleading "not valid JSON" parse failure.
        let resp = resp.error_for_status().map_err(|e| {
            not_entitled(format!(
                "the SQL interface JWKS registry returned an error status: {e}"
            ))
        })?;
        resp.text().map_err(|e| {
            not_entitled(format!(
                "unable to read the SQL interface JWKS response body: {e}"
            ))
        })
    }

    fn refresh(&self, cache: &mut JwksCache) -> Result<()> {
        let body = self.load_jwks_body()?;
        cache.keys = parse_jwks(&body)?;
        cache.fetched_at = Some(Instant::now());
        Ok(())
    }
}

impl Ed25519KeyProvider for JwksKeyProvider {
    fn verifying_key(&self, kid: &str) -> Result<VerifyingKey> {
        // Mutex is only poisoned if a holder panicked; recover the guard rather than propagating.
        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());

        let fresh = cache.fetched_at.is_some_and(|t| t.elapsed() < self.ttl);
        if fresh {
            if let Some(key) = cache.keys.get(kid) {
                return Ok(*key);
            }
        }

        // Either the cache is stale or the requested `kid` is unknown (possible mid-rotation):
        // re-fetch once. On fetch failure, fall back to any previously cached keys.
        if let Err(fetch_err) = self.refresh(&mut cache) {
            if let Some(key) = cache.keys.get(kid) {
                return Ok(*key);
            }
            return Err(fetch_err);
        }

        cache.keys.get(kid).copied().ok_or_else(|| {
            not_entitled(format!(
                "no SQL interface signing key found for key id '{kid}'; it may have been rotated out"
            ))
        })
    }
}

#[derive(Debug, Deserialize)]
struct JwtHeader {
    alg: String,
    #[serde(default)]
    kid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    #[serde(default)]
    iss: Option<String>,
    #[serde(default)]
    sub: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    exp: Option<i64>,
}

fn not_entitled(msg: impl Into<String>) -> Error {
    Error::SqlInterfaceNotEntitled(msg.into())
}

/// Extracts the Atlas dedicated cluster name from a `hello.me` value, if the host is an Atlas
/// dedicated cluster. Returns `None` for any other host (on-prem, self-managed, ADF, …), which
/// signals that the entitlement gate does not apply.
fn atlas_dedicated_cluster_name(me: &str) -> Option<String> {
    ATLAS_DEDICATED_HOST
        .captures(me)
        .map(|caps| caps[1].to_string())
}

/// Verifies the Atlas SQL Direct Cluster entitlement marker for this connection.
///
/// Returns `Ok(())` when the connection is allowed — either because the host is not an Atlas
/// dedicated cluster (gate does not apply) or because a valid, enabled marker was found. Returns
/// [`Error::SqlInterfaceNotEntitled`] with an actionable message otherwise.
pub(crate) async fn verify_sql_interface_entitlement(
    client: &Client,
    key_provider: &dyn Ed25519KeyProvider,
) -> Result<()> {
    // 1. Resolve the cluster name from `hello.me` and decide whether the gate applies.
    let admin_db = client.database("admin");
    let hello = run_command_with_retry(&admin_db, doc! { "hello": 1 })
        .await
        .map_err(Error::SqlInterfaceEntitlementCheckFailed)?;

    let cluster_name = match hello
        .get_str("me")
        .ok()
        .and_then(atlas_dedicated_cluster_name)
    {
        Some(name) => name,
        // Not an Atlas dedicated cluster (on-prem / self-managed Enterprise, or no resolvable
        // Atlas host). The entitlement gate does not apply; allow the connection.
        None => return Ok(()),
    };

    // 2. Read the singleton entitlement marker from the reserved collection.
    let marker_coll = client
        .database(MARKER_DB)
        .collection::<mongodb::bson::Document>(MARKER_COLLECTION);
    let marker = marker_coll
        .find_one(doc! { "_id": MARKER_ID })
        .await
        .map_err(Error::SqlInterfaceEntitlementCheckFailed)?;

    let token = match marker {
        Some(doc) => doc.get_str("token").map(str::to_string).map_err(|_| {
            not_entitled(
                "the Atlas SQL interface entitlement marker is malformed (missing 'token'); the \
                 SQL interface may not be enabled for this cluster",
            )
        })?,
        None => {
            return Err(not_entitled(
                "the SQL interface is not enabled for this Atlas cluster. Enable it for the \
                 cluster in Atlas before connecting.",
            ))
        }
    };

    // 3. Verify the compact JWS and validate its claims.
    verify_token(&token, &cluster_name, key_provider)
}

/// Verifies the compact-JWS marker string against the cluster name. Split out from the DB access
/// so it can be unit-tested directly.
fn verify_token(
    token: &str,
    cluster_name: &str,
    key_provider: &dyn Ed25519KeyProvider,
) -> Result<()> {
    let mut parts = token.split('.');
    let (header_b64, payload_b64, sig_b64) =
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(h), Some(p), Some(s), None) => (h, p, s),
            _ => {
                return Err(not_entitled(
                    "the Atlas SQL interface entitlement token is malformed (not a compact JWS)",
                ))
            }
        };

    let header: JwtHeader = decode_json_segment(header_b64, "header")?;
    let claims: JwtClaims = decode_json_segment(payload_b64, "payload")?;

    // Pin the algorithm before doing anything key-related.
    if header.alg != EXPECTED_ALG {
        return Err(not_entitled(format!(
            "the Atlas SQL interface entitlement token uses an unsupported algorithm '{}' \
             (expected {EXPECTED_ALG})",
            header.alg
        )));
    }

    let kid = header.kid.as_deref().ok_or_else(|| {
        not_entitled("the Atlas SQL interface entitlement token is missing a key id ('kid')")
    })?;

    // Verify the signature over `header.payload` using the key selected by `kid`.
    let verifying_key = key_provider.verifying_key(kid)?;
    let signature_bytes = URL_SAFE_NO_PAD.decode(sig_b64).map_err(|_| {
        not_entitled("the Atlas SQL interface entitlement token signature is not valid base64url")
    })?;
    let signature = Signature::from_slice(&signature_bytes).map_err(|_| {
        not_entitled("the Atlas SQL interface entitlement token signature has an invalid length")
    })?;
    let signing_input = format!("{header_b64}.{payload_b64}");
    verifying_key
        .verify(signing_input.as_bytes(), &signature)
        .map_err(|_| {
            not_entitled(
                "the Atlas SQL interface entitlement token signature is invalid; the token may \
                 have been tampered with or signed with a revoked key",
            )
        })?;

    // Validate the issuer.
    match claims.iss.as_deref() {
        Some(ISSUER_NORMAL) | Some(ISSUER_EMERGENCY) => {}
        other => {
            return Err(not_entitled(format!(
                "the Atlas SQL interface entitlement token has an unrecognized issuer ({})",
                other.unwrap_or("<missing>")
            )))
        }
    }

    // Validate expiry. Normal tokens carry no `exp` (valid until overwritten); emergency tokens
    // set one. We enforce `exp` only when present.
    if let Some(exp) = claims.exp {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|d| i64::try_from(d.as_secs()).ok())
            .unwrap_or(i64::MAX);
        if now >= exp {
            return Err(not_entitled(
                "the Atlas SQL interface entitlement token has expired. If this is an emergency \
                 access token, request a new one.",
            ));
        }
    }

    // Validate the subject matches this cluster (prevents copying a token between clusters).
    match claims.sub.as_deref() {
        Some(sub) if sub == cluster_name => {}
        _ => {
            return Err(not_entitled(
                "the Atlas SQL interface entitlement token was issued for a different cluster",
            ))
        }
    }

    // Finally, the feature must actually be ON.
    match claims.enabled {
        Some(true) => Ok(()),
        _ => Err(not_entitled(
            "the SQL interface is currently disabled for this Atlas cluster. Enable it for the \
             cluster in Atlas before connecting.",
        )),
    }
}

fn decode_json_segment<T: serde::de::DeserializeOwned>(segment: &str, which: &str) -> Result<T> {
    let bytes = URL_SAFE_NO_PAD.decode(segment).map_err(|_| {
        not_entitled(format!(
            "the Atlas SQL interface entitlement token {which} is not valid base64url"
        ))
    })?;
    serde_json::from_slice(&bytes).map_err(|_| {
        not_entitled(format!(
            "the Atlas SQL interface entitlement token {which} is not valid JSON"
        ))
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    const TEST_KID: &str = "test-kid";
    const CLUSTER: &str = "cluster0";

    struct StaticKeyProvider(VerifyingKey);
    impl Ed25519KeyProvider for StaticKeyProvider {
        fn verifying_key(&self, _kid: &str) -> Result<VerifyingKey> {
            Ok(self.0)
        }
    }

    fn signing_key() -> SigningKey {
        // Deterministic key for tests.
        SigningKey::from_bytes(&[7u8; 32])
    }

    /// Signs an arbitrary header/payload pair with the test key, producing a compact JWS. Kept
    /// generic so tests can vary the header (`alg`, `kid`) and omit individual claims.
    fn sign(header: serde_json::Value, payload: serde_json::Value) -> String {
        let h = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
        let p = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());
        let signing_input = format!("{h}.{p}");
        let sig = signing_key().sign(signing_input.as_bytes());
        format!("{signing_input}.{}", URL_SAFE_NO_PAD.encode(sig.to_bytes()))
    }

    fn test_header() -> serde_json::Value {
        serde_json::json!({ "alg": "EdDSA", "typ": "JWT", "kid": TEST_KID })
    }

    fn make_token(iss: &str, sub: &str, enabled: bool, exp: Option<i64>) -> String {
        let mut payload = serde_json::json!({
            "iss": iss,
            "sub": sub,
            "iat": 1_748_000_000i64,
            "enabled": enabled,
        });
        if let Some(exp) = exp {
            payload["exp"] = serde_json::json!(exp);
        }
        sign(test_header(), payload)
    }

    /// A well-formed payload for `CLUSTER` with every claim present, so tests can remove exactly
    /// one claim and attribute the resulting failure to that claim alone.
    fn full_payload() -> serde_json::Value {
        serde_json::json!({
            "iss": ISSUER_NORMAL,
            "sub": CLUSTER,
            "iat": 1_748_000_000i64,
            "enabled": true,
        })
    }

    fn payload_without(claim: &str) -> serde_json::Value {
        let mut payload = full_payload();
        payload.as_object_mut().unwrap().remove(claim);
        payload
    }

    fn error_message(err: Error) -> String {
        match err {
            Error::SqlInterfaceNotEntitled(msg) => msg,
            other => panic!("expected SqlInterfaceNotEntitled, got {other:?}"),
        }
    }

    fn provider() -> StaticKeyProvider {
        StaticKeyProvider(signing_key().verifying_key())
    }

    fn far_future() -> i64 {
        i64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
        .unwrap()
            + 3600
    }

    #[test]
    fn valid_normal_token_is_accepted() {
        let token = make_token(ISSUER_NORMAL, CLUSTER, true, None);
        assert!(verify_token(&token, CLUSTER, &provider()).is_ok());
    }

    #[test]
    fn valid_emergency_token_within_exp_is_accepted() {
        let token = make_token(ISSUER_EMERGENCY, CLUSTER, true, Some(far_future()));
        assert!(verify_token(&token, CLUSTER, &provider()).is_ok());
    }

    #[test]
    fn disabled_token_is_rejected() {
        let token = make_token(ISSUER_NORMAL, CLUSTER, false, None);
        assert!(verify_token(&token, CLUSTER, &provider()).is_err());
    }

    #[test]
    fn expired_emergency_token_is_rejected() {
        let token = make_token(ISSUER_EMERGENCY, CLUSTER, true, Some(1_000i64));
        assert!(verify_token(&token, CLUSTER, &provider()).is_err());
    }

    #[test]
    fn wrong_issuer_is_rejected() {
        let token = make_token("mongosql", CLUSTER, true, None);
        assert!(verify_token(&token, CLUSTER, &provider()).is_err());
    }

    #[test]
    fn wrong_subject_is_rejected() {
        let token = make_token(ISSUER_NORMAL, "other-cluster", true, None);
        assert!(verify_token(&token, CLUSTER, &provider()).is_err());
    }

    #[test]
    fn tampered_signature_is_rejected() {
        let mut token = make_token(ISSUER_NORMAL, CLUSTER, true, None);
        // Flip the last character of the signature segment.
        let last = token.pop().unwrap();
        token.push(if last == 'A' { 'B' } else { 'A' });
        assert!(verify_token(&token, CLUSTER, &provider()).is_err());
    }

    #[test]
    fn malformed_token_is_rejected() {
        assert!(verify_token("not-a-jwt", CLUSTER, &provider()).is_err());
        assert!(verify_token("a.b", CLUSTER, &provider()).is_err());
    }

    #[test]
    fn atlas_dedicated_hostnames_are_recognized() {
        assert_eq!(
            atlas_dedicated_cluster_name("cluster0-shard-00-00.abc123.mongodb.net:27017"),
            Some("cluster0".to_string())
        );
        assert_eq!(
            atlas_dedicated_cluster_name("cluster0-shard-00-mongos-g0.abc123.mongodb.net:27017"),
            Some("cluster0".to_string())
        );
    }

    #[test]
    fn parse_jwks_loads_ed25519_key_that_verifies() {
        let vk = signing_key().verifying_key();
        let x = URL_SAFE_NO_PAD.encode(vk.to_bytes());
        let body = serde_json::json!({
            "keys": [
                { "kty": "OKP", "crv": "Ed25519", "kid": TEST_KID, "use": "sig", "x": x }
            ]
        })
        .to_string();

        let keys = parse_jwks(&body).unwrap();
        assert!(keys.contains_key(TEST_KID));

        // The parsed key must actually verify a token signed by the matching private key.
        let provider = StaticKeyProvider(keys[TEST_KID]);
        let token = make_token(ISSUER_NORMAL, CLUSTER, true, None);
        assert!(verify_token(&token, CLUSTER, &provider).is_ok());
    }

    #[test]
    fn parse_jwks_ignores_non_ed25519_keys() {
        let body = serde_json::json!({
            "keys": [
                { "kty": "RSA", "kid": "rsa-1", "n": "abc", "e": "AQAB", "x": "unused" },
                { "kty": "OKP", "crv": "X25519", "kid": "x25519-1", "x": "unused" }
            ]
        })
        .to_string();
        assert!(parse_jwks(&body).unwrap().is_empty());
    }

    #[test]
    fn parse_jwks_rejects_malformed_key_material() {
        // OKP/Ed25519 entry whose `x` is not valid base64url.
        let body = serde_json::json!({
            "keys": [ { "kty": "OKP", "crv": "Ed25519", "kid": "bad", "x": "!!!notbase64!!!" } ]
        })
        .to_string();
        assert!(parse_jwks(&body).is_err());
    }

    // -----------------------------------------------------------------------------------------
    // Algorithm pinning (alg confusion). The `alg` header is attacker-controlled, so it must be
    // checked against EXPECTED_ALG before any key material is touched.
    // -----------------------------------------------------------------------------------------

    #[test]
    fn alg_none_is_rejected() {
        // The canonical alg-confusion attack: claim `none` and supply an empty signature.
        let header = serde_json::json!({ "alg": "none", "typ": "JWT", "kid": TEST_KID });
        let h = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
        let p = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&full_payload()).unwrap());
        let token = format!("{h}.{p}.");

        let msg = error_message(verify_token(&token, CLUSTER, &provider()).unwrap_err());
        assert!(
            msg.contains("unsupported algorithm"),
            "expected an algorithm rejection, got: {msg}"
        );
    }

    #[test]
    fn symmetric_alg_is_rejected() {
        // A validly Ed25519-signed token that *claims* HS256 must still be refused: we trust our
        // own pinned algorithm, never the token's self-declaration.
        let header = serde_json::json!({ "alg": "HS256", "typ": "JWT", "kid": TEST_KID });
        let token = sign(header, full_payload());

        let msg = error_message(verify_token(&token, CLUSTER, &provider()).unwrap_err());
        assert!(
            msg.contains("unsupported algorithm") && msg.contains("HS256"),
            "expected an algorithm rejection naming HS256, got: {msg}"
        );
    }

    #[test]
    fn missing_kid_is_rejected() {
        let header = serde_json::json!({ "alg": "EdDSA", "typ": "JWT" });
        let token = sign(header, full_payload());

        let msg = error_message(verify_token(&token, CLUSTER, &provider()).unwrap_err());
        assert!(
            msg.contains("missing a key id"),
            "expected a missing-kid rejection, got: {msg}"
        );
    }

    // -----------------------------------------------------------------------------------------
    // Absent claims. Every claim is `Option` with a serde default, so "absent" must fail closed
    // rather than being treated as a benign value.
    // -----------------------------------------------------------------------------------------

    #[test]
    fn missing_enabled_claim_is_rejected() {
        // Absent must behave like `false`, not like "unset, so allow".
        let token = sign(test_header(), payload_without("enabled"));
        let msg = error_message(verify_token(&token, CLUSTER, &provider()).unwrap_err());
        assert!(
            msg.contains("currently disabled"),
            "expected a disabled rejection, got: {msg}"
        );
    }

    #[test]
    fn missing_issuer_claim_is_rejected() {
        let token = sign(test_header(), payload_without("iss"));
        let msg = error_message(verify_token(&token, CLUSTER, &provider()).unwrap_err());
        assert!(
            msg.contains("unrecognized issuer") && msg.contains("<missing>"),
            "expected an issuer rejection reporting the claim as missing, got: {msg}"
        );
    }

    #[test]
    fn missing_subject_claim_is_rejected() {
        let token = sign(test_header(), payload_without("sub"));
        let msg = error_message(verify_token(&token, CLUSTER, &provider()).unwrap_err());
        assert!(
            msg.contains("different cluster"),
            "expected a subject rejection, got: {msg}"
        );
    }

    // -----------------------------------------------------------------------------------------
    // JwksKeyProvider cache behaviour. Driven through the `local_jwks_path` branch, which uses
    // the same fetch/parse/cache pipeline as the HTTP branch but needs no network.
    // -----------------------------------------------------------------------------------------

    /// Writes a JWKS document containing the test key to a unique temp path.
    fn write_test_jwks(name: &str) -> PathBuf {
        let vk = signing_key().verifying_key();
        let body = serde_json::json!({
            "keys": [{
                "kty": "OKP", "crv": "Ed25519", "kid": TEST_KID, "use": "sig",
                "x": URL_SAFE_NO_PAD.encode(vk.to_bytes()),
            }]
        })
        .to_string();

        let path = std::env::temp_dir().join(format!(
            "odbc-entitlement-{}-{name}.json",
            std::process::id()
        ));
        std::fs::write(&path, body).unwrap();
        path
    }

    /// Builds a local-file-backed provider with an explicit TTL, so cache staleness is
    /// deterministic instead of depending on wall-clock time.
    fn local_provider(path: Option<PathBuf>, ttl: Duration) -> JwksKeyProvider {
        JwksKeyProvider {
            // Unused: `local_jwks_path` short-circuits before any HTTP request is made.
            jwks_url: "http://127.0.0.1:1/unused".to_string(),
            local_jwks_path: path,
            http: reqwest::blocking::Client::new(),
            cache: Mutex::new(JwksCache::default()),
            ttl,
        }
    }

    #[test]
    fn jwks_provider_loads_keys_from_local_file() {
        let path = write_test_jwks("load");
        let provider = local_provider(Some(path.clone()), JWKS_CACHE_TTL);

        let key = provider.verifying_key(TEST_KID).unwrap();
        assert_eq!(key.to_bytes(), signing_key().verifying_key().to_bytes());

        // The key must actually verify a real marker, not just parse.
        let token = make_token(ISSUER_NORMAL, CLUSTER, true, None);
        assert!(verify_token(&token, CLUSTER, &provider).is_ok());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn jwks_provider_serves_cached_key_within_ttl() {
        let path = write_test_jwks("cached");
        let provider = local_provider(Some(path.clone()), JWKS_CACHE_TTL);

        provider.verifying_key(TEST_KID).unwrap();
        // Remove the source: a fresh cache must not re-read it.
        std::fs::remove_file(&path).unwrap();
        assert!(provider.verifying_key(TEST_KID).is_ok());
    }

    #[test]
    fn jwks_provider_falls_back_to_stale_keys_when_refetch_fails() {
        let path = write_test_jwks("stale");
        // A zero TTL makes the cache stale on every lookup, forcing a re-fetch each time.
        let provider = local_provider(Some(path.clone()), Duration::ZERO);

        provider.verifying_key(TEST_KID).unwrap();
        std::fs::remove_file(&path).unwrap();

        // Re-fetch now fails, but a previously cached key is better than refusing the connection.
        assert!(provider.verifying_key(TEST_KID).is_ok());
    }

    #[test]
    fn jwks_provider_reports_unknown_kid_as_possible_rotation() {
        let path = write_test_jwks("unknown-kid");
        let provider = local_provider(Some(path.clone()), JWKS_CACHE_TTL);

        // An unknown `kid` triggers a re-fetch (mid-rotation); the key is still absent afterwards.
        let msg = error_message(provider.verifying_key("no-such-kid").unwrap_err());
        assert!(
            msg.contains("rotated out") && msg.contains("no-such-kid"),
            "expected a rotated-out error naming the kid, got: {msg}"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn jwks_provider_surfaces_fetch_error_when_nothing_is_cached() {
        let missing = std::env::temp_dir().join("odbc-entitlement-does-not-exist.json");
        let _ = std::fs::remove_file(&missing);
        let provider = local_provider(Some(missing), JWKS_CACHE_TTL);

        let msg = error_message(provider.verifying_key(TEST_KID).unwrap_err());
        assert!(
            msg.contains("unable to read the local SQL interface public key file"),
            "expected the local-file read error, got: {msg}"
        );
    }

    #[test]
    fn non_atlas_hostnames_are_not_gated() {
        assert_eq!(
            atlas_dedicated_cluster_name("my-onprem-host.example.com:27017"),
            None
        );
        assert_eq!(atlas_dedicated_cluster_name("localhost:27017"), None);
        // ADF-style hosts have no `-shard-` segment.
        assert_eq!(
            atlas_dedicated_cluster_name("mycluster-abc123.a.query.mongodb.net"),
            None
        );
    }
}

//! SQL-3340: Atlas SQL Direct Cluster SQL Interface status gate.
//!
//! Atlas SQL "Direct Cluster" is a per-cluster toggle. When it is enabled, the Atlas control plane
//! writes a status marker — a compact JWS (signed JWT) — into the reserved
//! `__mdb_internal_sqlinterface.__sql_status` collection on the user cluster. The driver inspects
//! that marker when a logical connection is established and refuses to connect when the SQL
//! interface is not enabled for the cluster.
//!
//! **Signature verification is descoped for GA.** Markers are still signed so that signature
//! verification ("fingerprinting") can be added in a later milestone, but this gate only
//! base64url-decodes the payload and evaluates the status-bearing claims. See
//! `AtlasSQLDirectCluster_marker.md` and `AtlasSQLDirectCluster_marker_fingerprinting.md` in
//! 10gen/engineering-documents. This mirrors the schema-manager gate added in
//! 10gen/schema-manager-rs#1014.
//!
//! Scope: the gate applies **only to Atlas dedicated clusters**. On-prem / self-managed Enterprise
//! deployments have no marker and must still be allowed to connect, so the cluster name is derived
//! from `hello.me` (see [`atlas_dedicated_cluster_name`]) and the gate is skipped when the host is
//! not an Atlas dedicated host.

use crate::{err::Result, Error};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use mongodb::{bson::doc, bson::Document, Client};
use serde::Deserialize;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Issuer (`iss`) for normal markers written by the CNCP SQL Interface service.
pub const ISSUER_NORMAL: &str = "mongosql-service";
/// Issuer (`iss`) for break-glass markers written by the restricted support tool (SQL-3337).
/// These are short-lived and MUST carry a valid `exp`.
pub const ISSUER_EMERGENCY: &str = "mongosql-emergency";

/// Reserved database holding the SQL interface status marker.
const MARKER_DB: &str = "__mdb_internal_sqlinterface";
/// Reserved collection holding the SQL interface status marker.
const MARKER_COLLECTION: &str = "__sql_status";
/// Singleton document `_id` for the status marker.
const MARKER_ID: &str = "entitlement";

// Atlas dedicated cluster hostnames always embed the cluster name before the first `-shard-`
// segment and resolve under `.mongodb.net`, e.g.
//   cluster0-shard-00-00.abc123.mongodb.net:27017        (replica set node)
//   cluster0-shard-00-mongos-g0.abc123.mongodb.net:27017 (mongos)
// The cluster name is the prefix before `-shard-`. On-prem / self-managed hosts do not match,
// which is exactly how we scope the status gate to Atlas dedicated clusters only. We require the
// `.mongodb.net` suffix rather than just looking for `-shard-`, so that a self-managed host that
// happens to be named `<something>-shard-0.<domain>` is not mistaken for an Atlas cluster and
// wrongly gated.
static ATLAS_DEDICATED_HOST: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^([^.]+)-shard-\d+.*\.mongodb\.net(:\d+)?$").unwrap());

/// The status-bearing claims of the marker payload. Only the fields this gate inspects are
/// modeled; every field is optional so a marker missing any of them is a status validation
/// failure rather than a deserialization failure. `exp` is epoch seconds, present only on
/// emergency markers.
#[derive(Debug, Deserialize)]
struct MarkerClaims {
    iss: Option<String>,
    sub: Option<String>,
    enabled: Option<bool>,
    exp: Option<i64>,
}

/// Extracts the Atlas dedicated cluster name from a `hello.me` value, if the host is an Atlas
/// dedicated cluster. Returns `None` for any other host (on-prem, self-managed, ADF, …), which
/// signals that the status gate does not apply.
///
/// The name is lowercased to match the producer's convention for the marker's `sub` claim. Atlas
/// hostnames are already lowercase; this is defensive normalization.
pub fn atlas_dedicated_cluster_name(me: &str) -> Option<String> {
    ATLAS_DEDICATED_HOST
        .captures(me)
        .map(|caps| caps[1].to_lowercase())
}

/// Runs `hello` and derives the Atlas dedicated cluster name from its `me` field, returning `None`
/// when the host is not an Atlas dedicated cluster.
///
/// This is done once during mongo connection setup (when the `Client` is created) and the result
/// is cached alongside the client, so the per-logical-connection gate does not repeat it. A `hello`
/// reply with no usable `me` field yields `None`: `me` is absent on some standalone / direct
/// connection topologies, and treating that as "not an Atlas dedicated cluster" is the same allow
/// decision we make for any other non-Atlas host.
pub async fn resolve_atlas_cluster_name(client: &Client) -> Result<Option<String>> {
    let hello = crate::util::run_command_with_retry(&client.database("admin"), doc! { "hello": 1 })
        .await
        .map_err(Error::SqlInterfaceStatusReadFailed)?;

    let name = hello
        .get_str("me")
        .ok()
        .and_then(atlas_dedicated_cluster_name);

    if let Some(ref name) = name {
        log::info!("resolved Atlas cluster name {name:?}");
    }
    Ok(name)
}

/// Fail-closed SQL interface status gate, run when a logical connection is established.
///
/// `atlas_cluster_name` is the name resolved during connection setup by
/// [`resolve_atlas_cluster_name`]. `None` means the host is not an Atlas dedicated cluster, so the
/// gate does not apply and the connection is allowed.
///
/// For an Atlas dedicated cluster this reads the marker and inspects its claims **without**
/// validating the signature: the issuer must be recognized, `sub` must equal the cluster name, and
/// the status must be explicitly enabled (emergency markers additionally require an unexpired
/// `exp`). Any other state is rejected.
pub async fn verify_sql_interface_enabled(
    client: &Client,
    atlas_cluster_name: Option<&str>,
) -> Result<()> {
    let Some(cluster_name) = atlas_cluster_name else {
        log::info!("cluster is not an Atlas dedicated cluster; SQL interface gate not applicable");
        return Ok(());
    };

    let token = read_marker_token(client).await?;
    let claims = decode_marker_claims(&token).ok_or_else(|| {
        log::warn!("marker payload could not be decoded");
        Error::SqlInterfaceStatusInvalid
    })?;
    evaluate_claims(&claims, cluster_name)
}

/// Reads the marker token from `__sql_status` using a Primary read, so the gate never observes a
/// stale replica and always sees the latest toggle state. A read failure is
/// [`Error::SqlInterfaceStatusReadFailed`]; a missing document is
/// [`Error::SqlInterfaceUnavailable`]; a present document with an empty or non-string token is
/// [`Error::SqlInterfaceStatusInvalid`].
async fn read_marker_token(client: &Client) -> Result<String> {
    let marker = client
        .database(MARKER_DB)
        .collection::<Document>(MARKER_COLLECTION)
        .find_one(doc! { "_id": MARKER_ID })
        .selection_criteria(mongodb::options::SelectionCriteria::ReadPreference(
            mongodb::options::ReadPreference::Primary,
        ))
        .await
        .map_err(|e| {
            log::warn!("reading the SQL interface status marker failed: {e}");
            Error::SqlInterfaceStatusReadFailed(e)
        })?
        .ok_or_else(|| {
            log::warn!("no SQL interface status marker found for this cluster");
            Error::SqlInterfaceUnavailable
        })?;

    match marker.get_str("token") {
        Ok(token) if !token.is_empty() => Ok(token.to_string()),
        _ => {
            log::warn!("SQL interface status marker has no readable token field");
            Err(Error::SqlInterfaceStatusInvalid)
        }
    }
}

/// Extracts the status-bearing claims from a compact JWS by base64url-decoding its payload (the
/// second segment), per RFC 4648 §5 (no padding). The signature is neither required nor validated,
/// per the status-only gate.
fn decode_marker_claims(token: &str) -> Option<MarkerClaims> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload.as_bytes()).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Current time as epoch seconds, used to check emergency-marker expiry. Returns `None` if the
/// system clock is at or before the Unix epoch, so a misconfigured or tampered clock fails closed
/// rather than silently skewing expiry validation.
fn now_epoch_seconds() -> Option<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
}

/// Evaluates the decoded marker claims against the connected cluster's lowercase canonical name,
/// without validating the signature. Only an explicit `enabled: true` passes: an explicit
/// `enabled: false` is [`Error::SqlInterfaceDisabled`], and any unrecognized issuer,
/// mismatched/missing `sub`, missing `enabled`, or invalid emergency `exp` is
/// [`Error::SqlInterfaceStatusInvalid`].
fn evaluate_claims(claims: &MarkerClaims, cluster_name: &str) -> Result<()> {
    let issuer = match claims.iss.as_deref() {
        Some(issuer @ (ISSUER_NORMAL | ISSUER_EMERGENCY)) => issuer,
        other => {
            log::warn!("unrecognized SQL interface marker issuer {other:?}");
            return Err(Error::SqlInterfaceStatusInvalid);
        }
    };

    // `sub` binds the marker to this cluster, preventing a valid marker from being copied from
    // another cluster.
    if claims.sub.as_deref() != Some(cluster_name) {
        log::warn!("cluster identity validation failed for {:?}", claims.sub);
        return Err(Error::SqlInterfaceStatusInvalid);
    }

    // Normal markers carry no `exp` and are valid until overwritten. Emergency markers are
    // short-lived and must carry a present, unexpired `exp`.
    if issuer == ISSUER_EMERGENCY {
        match (claims.exp, now_epoch_seconds()) {
            (_, None) => {
                log::warn!("system clock appears to be before the Unix epoch; cannot validate exp");
                return Err(Error::SqlInterfaceStatusInvalid);
            }
            (Some(exp), Some(now)) if exp > now => {}
            _ => {
                log::warn!(
                    "emergency SQL interface marker for cluster {cluster_name:?} has a missing or \
                     expired exp"
                );
                return Err(Error::SqlInterfaceStatusInvalid);
            }
        }
    }

    match claims.enabled {
        Some(true) => Ok(()),
        Some(false) => {
            log::warn!("SQL interface is disabled for cluster {cluster_name:?}");
            Err(Error::SqlInterfaceDisabled)
        }
        None => {
            log::warn!("SQL interface marker for cluster {cluster_name:?} has no enabled flag");
            Err(Error::SqlInterfaceStatusInvalid)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    /// Builds a compact JWS with an arbitrary header/signature and the given claims as the
    /// payload. The header alg and the signature are deliberately junk: the status-only gate must
    /// never look at either.
    fn token_with_claims(claims: &serde_json::Value) -> String {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"Ed25519","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(claims.to_string().as_bytes());
        format!("{header}.{payload}.c2lnbmF0dXJl")
    }

    fn claims(iss: Option<&str>, sub: Option<&str>, enabled: Option<bool>) -> MarkerClaims {
        MarkerClaims {
            iss: iss.map(str::to_string),
            sub: sub.map(str::to_string),
            enabled,
            exp: None,
        }
    }

    /// Epoch seconds offset from now, for exercising emergency-marker expiry.
    fn epoch_from_now(offset_secs: i64) -> i64 {
        now_epoch_seconds().expect("system clock is after the Unix epoch") + offset_secs
    }

    #[test]
    fn decode_marker_claims_decodes_external_reference_payload() {
        // The payload segment was produced independently by Python's base64.urlsafe_b64encode
        // (padding stripped), proving we decode a real externally-encoded token, not just our own
        // encoder. Header/signature segments are irrelevant to the status-only gate.
        let token = format!(
            "aaa.{}.bbb",
            "eyJpc3MiOiJtb25nb3NxbC1zZXJ2aWNlIiwic3ViIjoiam9uYXRoYW50ZXN0Y2x1c3RlciIsImlhdCI6MTc4NzMzMzg1NCwiZW5hYmxlZCI6dHJ1ZX0"
        );
        let decoded = decode_marker_claims(&token).expect("reference payload decodes");
        assert_eq!(decoded.iss.as_deref(), Some(ISSUER_NORMAL));
        assert_eq!(decoded.sub.as_deref(), Some("jonathantestcluster"));
        assert_eq!(decoded.enabled, Some(true));
    }

    #[test]
    fn decode_marker_claims_extracts_payload_segment() {
        let token = token_with_claims(&json!({
            "iss": ISSUER_NORMAL,
            "sub": "cluster0",
            "enabled": true,
        }));
        let decoded = decode_marker_claims(&token).expect("payload decodes");
        assert_eq!(decoded.iss.as_deref(), Some(ISSUER_NORMAL));
        assert_eq!(decoded.sub.as_deref(), Some("cluster0"));
        assert_eq!(decoded.enabled, Some(true));
    }

    #[test]
    fn decode_marker_claims_rejects_malformed_input() {
        // Fewer than two segments.
        assert!(decode_marker_claims("not-a-jws").is_none());
        // A middle segment that is not valid base64url.
        assert!(decode_marker_claims("aaa.!!!.bbb").is_none());
        // A middle segment that decodes but is not a JSON object.
        assert!(
            decode_marker_claims(&format!("aaa.{}.bbb", URL_SAFE_NO_PAD.encode(b"{"))).is_none()
        );
    }

    #[test]
    fn no_signature_validation_is_attempted() {
        // The signature segment is garbage and the header alg is non-standard, yet an enabled
        // marker with a matching issuer and sub still passes: the gate never inspects either.
        let token = token_with_claims(&json!({
            "iss": ISSUER_NORMAL,
            "sub": "cluster0",
            "enabled": true,
        }));
        let decoded = decode_marker_claims(&token).expect("payload decodes");
        assert!(evaluate_claims(&decoded, "cluster0").is_ok());
    }

    #[test]
    fn enabled_marker_with_matching_issuer_and_sub_passes() {
        let c = claims(Some(ISSUER_NORMAL), Some("cluster0"), Some(true));
        assert!(evaluate_claims(&c, "cluster0").is_ok());
    }

    #[test]
    fn explicitly_disabled_marker_is_disabled() {
        let c = claims(Some(ISSUER_NORMAL), Some("cluster0"), Some(false));
        assert!(matches!(
            evaluate_claims(&c, "cluster0"),
            Err(Error::SqlInterfaceDisabled)
        ));
    }

    #[test]
    fn enabled_emergency_marker_with_unexpired_exp_passes() {
        let mut c = claims(Some(ISSUER_EMERGENCY), Some("cluster0"), Some(true));
        c.exp = Some(epoch_from_now(3_600));
        assert!(evaluate_claims(&c, "cluster0").is_ok());
    }

    #[test]
    fn emergency_marker_without_exp_is_invalid() {
        // Emergency markers are short-lived and must carry an exp; absence is rejected.
        let c = claims(Some(ISSUER_EMERGENCY), Some("cluster0"), Some(true));
        assert!(matches!(
            evaluate_claims(&c, "cluster0"),
            Err(Error::SqlInterfaceStatusInvalid)
        ));
    }

    #[test]
    fn expired_emergency_marker_is_invalid() {
        let mut c = claims(Some(ISSUER_EMERGENCY), Some("cluster0"), Some(true));
        c.exp = Some(epoch_from_now(-1));
        assert!(matches!(
            evaluate_claims(&c, "cluster0"),
            Err(Error::SqlInterfaceStatusInvalid)
        ));
    }

    #[test]
    fn normal_marker_ignores_exp() {
        // exp is not part of the normal-marker contract, so a present (even expired) exp is
        // ignored.
        let mut c = claims(Some(ISSUER_NORMAL), Some("cluster0"), Some(true));
        c.exp = Some(epoch_from_now(-1));
        assert!(evaluate_claims(&c, "cluster0").is_ok());
    }

    #[test]
    fn missing_enabled_flag_is_invalid() {
        let c = claims(Some(ISSUER_NORMAL), Some("cluster0"), None);
        assert!(matches!(
            evaluate_claims(&c, "cluster0"),
            Err(Error::SqlInterfaceStatusInvalid)
        ));
    }

    #[test]
    fn unrecognized_or_missing_issuer_is_invalid() {
        let wrong = claims(Some("attacker"), Some("cluster0"), Some(true));
        assert!(matches!(
            evaluate_claims(&wrong, "cluster0"),
            Err(Error::SqlInterfaceStatusInvalid)
        ));
        let missing = claims(None, Some("cluster0"), Some(true));
        assert!(matches!(
            evaluate_claims(&missing, "cluster0"),
            Err(Error::SqlInterfaceStatusInvalid)
        ));
    }

    #[test]
    fn mismatched_or_missing_sub_is_invalid() {
        let mismatched = claims(Some(ISSUER_NORMAL), Some("other"), Some(true));
        assert!(matches!(
            evaluate_claims(&mismatched, "cluster0"),
            Err(Error::SqlInterfaceStatusInvalid)
        ));
        let missing = claims(Some(ISSUER_NORMAL), None, Some(true));
        assert!(matches!(
            evaluate_claims(&missing, "cluster0"),
            Err(Error::SqlInterfaceStatusInvalid)
        ));
    }

    #[test]
    fn sub_comparison_is_case_sensitive_to_lowercase_canonical_name() {
        // The producer emits the lowercase canonical name; a display-cased sub must not match.
        let c = claims(Some(ISSUER_NORMAL), Some("Cluster0"), Some(true));
        assert!(matches!(
            evaluate_claims(&c, "cluster0"),
            Err(Error::SqlInterfaceStatusInvalid)
        ));
    }

    #[test]
    fn derives_cluster_name_from_replica_set_host() {
        assert_eq!(
            atlas_dedicated_cluster_name("cluster0-shard-00-00.abc123.mongodb.net:27017"),
            Some("cluster0".to_string())
        );
    }

    #[test]
    fn derives_cluster_name_from_mongos_host() {
        assert_eq!(
            atlas_dedicated_cluster_name("cluster0-shard-00-mongos-g0.abc123.mongodb.net:27017"),
            Some("cluster0".to_string())
        );
    }

    #[test]
    fn derives_cluster_name_without_port() {
        assert_eq!(
            atlas_dedicated_cluster_name("cluster0-shard-00-00.abc123.mongodb.net"),
            Some("cluster0".to_string())
        );
    }

    #[test]
    fn derive_cluster_name_lowercases_to_match_producer_convention() {
        // Atlas hosts are lowercase, but normalize defensively so a mixed-case host still matches
        // the producer's lowercase `sub`.
        assert_eq!(
            atlas_dedicated_cluster_name("MyCluster-shard-00-00.abc123.mongodb.net:27017"),
            Some("mycluster".to_string())
        );
    }

    #[test]
    fn derive_cluster_name_rejects_non_atlas_hosts() {
        // On-prem / self-managed hosts must not be gated.
        assert_eq!(atlas_dedicated_cluster_name("localhost:27017"), None);
        assert_eq!(
            atlas_dedicated_cluster_name("mongod-0.internal.example.com:27017"),
            None
        );
        // A self-managed host that merely looks shard-ish is not an Atlas host: the
        // `.mongodb.net` suffix is required.
        assert_eq!(
            atlas_dedicated_cluster_name("foo-shard-0.internal.example.com:27017"),
            None
        );
    }

    #[test]
    fn derive_cluster_name_rejects_empty_name() {
        assert_eq!(
            atlas_dedicated_cluster_name("-shard-00-00.abc123.mongodb.net"),
            None
        );
    }
}

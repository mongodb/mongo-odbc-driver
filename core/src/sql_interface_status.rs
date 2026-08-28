use crate::{err::Result, Error};
use data_encoding::BASE64URL_NOPAD;
use mongodb::{
    bson::{doc, from_document, Document},
    Client,
};
use serde::Deserialize;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Issuer (`iss`) for normal markers written by the CNCP SQL Interface service.
pub const ISSUER_NORMAL: &str = "mongosql-service";
/// Issuer (`iss`) for break-glass markers written by the restricted support tool.
/// These are short-lived and MUST carry a valid `exp`.
pub const ISSUER_EMERGENCY: &str = "mongosql-emergency";

/// Reserved database holding the SQL interface status marker.
const MARKER_DB: &str = "__mdb_internal_sqlinterface";
/// Reserved collection holding the SQL interface status marker.
const MARKER_COLLECTION: &str = "__sql_status";
/// Singleton document `_id` for the status marker.
const MARKER_ID: &str = "entitlement";

static ATLAS_DEDICATED_HOST: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^([^.]+)-shard-\d+.*\.mongodb\.net(?::\d+)?$")
        .expect("the Atlas dedicated host pattern should be a valid regex")
});

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
    ATLAS_DEDICATED_HOST.captures(me).map(|caps| {
        // A missing group here would mean the pattern above lost its capture group, not that this
        // host is non-Atlas. Panicking is deliberate: it is caught at the ODBC boundary and fails
        // the connection, whereas returning `None` would be read as "not an Atlas cluster" and
        // would silently disable the gate for every Atlas cluster.
        caps.get(1)
            .expect("a matched Atlas dedicated host always has a cluster-name capture group")
            .as_str()
            .to_lowercase()
    })
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

/// The marker document stored in `__sql_status`. Only `token` is consumed here; `_id` and the
/// producer's `timestamp` field are ignored.
///
/// This is deserialized from a fetched [`Document`] rather than by typing the collection itself,
/// so that a malformed marker is reported as [`Error::SqlInterfaceStatusInvalid`] rather than
/// arriving on `find_one`'s error channel and being misreported as a read failure.
#[derive(Debug, Deserialize)]
struct MarkerDoc {
    token: String,
}

/// Reads the marker token from `__sql_status` using a Primary read, so the gate never observes a
/// stale replica and always sees the latest toggle state. A read failure is
/// [`Error::SqlInterfaceStatusReadFailed`]; a missing document is
/// [`Error::SqlInterfaceUnavailable`]; a present document whose `token` is missing, not a string,
/// or empty is [`Error::SqlInterfaceStatusInvalid`].
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

    let marker: MarkerDoc = from_document(marker).map_err(|e| {
        log::warn!("SQL interface status marker has no readable token field: {e}");
        Error::SqlInterfaceStatusInvalid
    })?;

    if marker.token.is_empty() {
        log::warn!("SQL interface status marker has an empty token field");
        return Err(Error::SqlInterfaceStatusInvalid);
    }

    Ok(marker.token)
}

/// Extracts the status-bearing claims from a compact JWS by base64url-decoding its payload (the
/// second segment), per RFC 4648 §5 (no padding). The signature segment must be present but is
/// neither decoded nor validated, per the status-only gate.
///
/// The token must have exactly three segments. Requiring the signature segment does not
/// authenticate anything — we do not verify it — but it keeps a truncated or otherwise malformed
/// marker from being honored as if it were well formed, which matches the gate's fail-closed
/// stance and the shape a signature-verifying gate will require.
fn decode_marker_claims(token: &str) -> Option<MarkerClaims> {
    let segments: Vec<&str> = token.split('.').collect();
    let [_header, payload, _signature] = segments[..] else {
        return None;
    };
    let bytes = BASE64URL_NOPAD.decode(payload.as_bytes()).ok()?;
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
mod test;

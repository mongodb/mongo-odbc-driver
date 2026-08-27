use super::*;
use serde_json::json;

/// Builds a compact JWS with an arbitrary header/signature and the given claims as the
/// payload. The header alg and the signature are deliberately junk: the status-only gate must
/// never look at either.
fn token_with_claims(claims: &serde_json::Value) -> String {
    let header = BASE64URL_NOPAD.encode(br#"{"alg":"Ed25519","typ":"JWT"}"#);
    let payload = BASE64URL_NOPAD.encode(claims.to_string().as_bytes());
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
    assert!(decode_marker_claims(&format!("aaa.{}.bbb", BASE64URL_NOPAD.encode(b"{"))).is_none());
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
    // The port separator is required when a port is present: the `:` is a literal inside the
    // optional group, so digits may not be glued directly onto the hostname.
    assert_eq!(
        atlas_dedicated_cluster_name("cluster0-shard-00-00.abc123.mongodb.net27017"),
        None
    );
    // A non-numeric port does not match either.
    assert_eq!(
        atlas_dedicated_cluster_name("cluster0-shard-00-00.abc123.mongodb.net:notaport"),
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

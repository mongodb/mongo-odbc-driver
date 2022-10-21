use odbc_sys::ConnectionAttribute;

pub(crate) fn connection_attribute_to_string(attr: ConnectionAttribute) -> String {
    match attr {
        ConnectionAttribute::AsyncEnable => "ASYNC_ENABLE".to_string(),
        ConnectionAttribute::AccessMode => "ACCESS_MODE".to_string(),
        ConnectionAttribute::AutoCommit => "AUTO_COMMIT".to_string(),
        ConnectionAttribute::LoginTimeout => "LOGIN_TIMEOUT".to_string(),
        ConnectionAttribute::Trace => "TRACE".to_string(),
        ConnectionAttribute::TraceFile => "TRACE_FILE".to_string(),
        ConnectionAttribute::TranslateLib => "TRANSLATE_LIB".to_string(),
        ConnectionAttribute::TranslateOption => "TRANSLATE_OPTION".to_string(),
        ConnectionAttribute::TxnIsolation => "TXN_ISOLATION".to_string(),
        ConnectionAttribute::CurrentCatalog => "CURRENT_CATALOG".to_string(),
        ConnectionAttribute::OdbcCursors => "ODBC_CURSORS".to_string(),
        ConnectionAttribute::QuietMode => "QUIET_MODE".to_string(),
        ConnectionAttribute::PacketSize => "PACKET_SIZE".to_string(),
        ConnectionAttribute::ConnectionTimeout => "CONNECTION_TIMEOUT".to_string(),
        ConnectionAttribute::DisconnectBehaviour => "DISCONNECT_BEHAVIOUR".to_string(),
        ConnectionAttribute::AsyncDbcFunctionsEnable => "ASYNC_DBC_FUNCTIONS_ENABLE".to_string(),
        ConnectionAttribute::AsyncDbcEvent => "ASYNC_DBC_EVENT".to_string(),
        ConnectionAttribute::EnlistInDtc => "ENLIST_IN_DTC".to_string(),
        ConnectionAttribute::EnlistInXa => "ENLIST_IN_XA".to_string(),
        ConnectionAttribute::ConnectionDead => "CONNECTION_DEAD".to_string(),
        ConnectionAttribute::AutoIpd => "AUTO_IPD".to_string(),
        ConnectionAttribute::MetadataId => "METADATA_ID".to_string(),
    }
}

use definitions::{AttrOdbcVersion, ConnectionAttribute, SqlDataType, StatementAttribute};

pub(crate) fn connection_attribute_to_string(attr: ConnectionAttribute) -> String {
    match attr {
        ConnectionAttribute::SQL_ATTR_ASYNC_ENABLE => "ASYNC_ENABLE".to_string(),
        ConnectionAttribute::SQL_ATTR_ACCESS_MODE => "ACCESS_MODE".to_string(),
        ConnectionAttribute::SQL_ATTR_AUTOCOMMIT => "AUTO_COMMIT".to_string(),
        ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT => "LOGIN_TIMEOUT".to_string(),
        ConnectionAttribute::SQL_ATTR_TRACE => "TRACE".to_string(),
        ConnectionAttribute::SQL_ATTR_TRACEFILE => "TRACE_FILE".to_string(),
        ConnectionAttribute::SQL_ATTR_TRANSLATE_LIB => "TRANSLATE_LIB".to_string(),
        ConnectionAttribute::SQL_ATTR_TRANSLATE_OPTION => "TRANSLATE_OPTION".to_string(),
        ConnectionAttribute::SQL_ATTR_TXN_ISOLATION => "TXN_ISOLATION".to_string(),
        ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG => "CURRENT_CATALOG".to_string(),
        ConnectionAttribute::SQL_ATTR_ODBC_CURSORS => "ODBC_CURSORS".to_string(),
        ConnectionAttribute::SQL_ATTR_QUIET_MODE => "QUIET_MODE".to_string(),
        ConnectionAttribute::SQL_ATTR_PACKET_SIZE => "PACKET_SIZE".to_string(),
        ConnectionAttribute::SQL_ATTR_CONNECTION_TIMEOUT => "CONNECTION_TIMEOUT".to_string(),
        ConnectionAttribute::SQL_ATTR_DISCONNECT_BEHAVIOR => "DISCONNECT_BEHAVIOUR".to_string(),
        ConnectionAttribute::SQL_ATTR_ASYNC_DBC_FUNCTIONS_ENABLE => {
            "ASYNC_DBC_FUNCTIONS_ENABLE".to_string()
        }
        ConnectionAttribute::SQL_ATTR_ASYNC_DBC_EVENT => "ASYNC_DBC_EVENT".to_string(),
        ConnectionAttribute::SQL_ATTR_ENLIST_IN_DTC => "ENLIST_IN_DTC".to_string(),
        ConnectionAttribute::SQL_ATTR_ENLIST_IN_XA => "ENLIST_IN_XA".to_string(),
        ConnectionAttribute::SQL_ATTR_CONNECTION_DEAD => "CONNECTION_DEAD".to_string(),
        ConnectionAttribute::SQL_ATTR_AUTO_IPD => "AUTO_IPD".to_string(),
        ConnectionAttribute::SQL_ATTR_METADATA_ID => "METADATA_ID".to_string(),
        ConnectionAttribute::SQL_ATTR_APP_WCHAR_TYPE => "APP_WCHAR_TYPE".to_string(),
    }
}

pub(crate) fn statement_attribute_to_string(attr: StatementAttribute) -> String {
    match attr {
        StatementAttribute::SQL_ROWSET_SIZE => "ROWSET_SIZE".to_string(),
        StatementAttribute::SQL_GET_BOOKMARK => "GET_BOOKMARK".to_string(),
        StatementAttribute::SQL_ATTR_APP_ROW_DESC => "APP_ROW_DESC".to_string(),
        StatementAttribute::SQL_ATTR_APP_PARAM_DESC => "APP_PARAM_DESC".to_string(),
        StatementAttribute::SQL_ATTR_IMP_ROW_DESC => "IMP_ROW_DESC".to_string(),
        StatementAttribute::SQL_ATTR_IMP_PARAM_DESC => "IMP_PARAM_DESC".to_string(),
        StatementAttribute::SQL_ATTR_CURSOR_SCROLLABLE => "CURSOR_SCROLLABLE".to_string(),
        StatementAttribute::SQL_ATTR_CURSOR_SENSITIVITY => "CURSOR_SENSITIVITY".to_string(),
        StatementAttribute::SQL_ATTR_ASYNC_ENABLE => "ASYNC_ENABLE".to_string(),
        StatementAttribute::SQL_ATTR_CONCURRENCY => "CONCURRENCY".to_string(),
        StatementAttribute::SQL_ATTR_CURSOR_TYPE => "CURSOR_TYPE".to_string(),
        StatementAttribute::SQL_ATTR_ENABLE_AUTO_IPD => "ENABLE_AUTO_IPD".to_string(),
        StatementAttribute::SQL_ATTR_FETCH_BOOKMARK_PTR => "FETCH_BOOKMARK_PTR".to_string(),
        StatementAttribute::SQL_ATTR_KEYSET_SIZE => "KEYSET_SIZE".to_string(),
        StatementAttribute::SQL_ATTR_MAX_LENGTH => "MAX_LENGTH".to_string(),
        StatementAttribute::SQL_ATTR_MAX_ROWS => "MAX_ROWS".to_string(),
        StatementAttribute::SQL_ATTR_NOSCAN => "NOSCAN".to_string(),
        StatementAttribute::SQL_ATTR_PARAM_BIND_OFFSET_PTR => "PARAM_BIND_OFFSET_PTR".to_string(),
        StatementAttribute::SQL_ATTR_PARAM_BIND_TYPE => "PARAM_BIND_TYPE".to_string(),
        StatementAttribute::SQL_ATTR_PARAM_OPERATION_PTR => "PARAM_OPERATION_PTR".to_string(),
        StatementAttribute::SQL_ATTR_PARAM_STATUS_PTR => "PARAM_STATUS_PTR".to_string(),
        StatementAttribute::SQL_ATTR_PARAMS_PROCESSED_PTR => "PARAMS_PROCESSED_PTR".to_string(),
        StatementAttribute::SQL_ATTR_PARAMSET_SIZE => "PARAMSET_SIZE".to_string(),
        StatementAttribute::SQL_ATTR_QUERY_TIMEOUT => "QUERY_TIMEOUT".to_string(),
        StatementAttribute::SQL_ATTR_RETRIEVE_DATA => "RETRIEVE_DATA".to_string(),
        StatementAttribute::SQL_ATTR_ROW_BIND_OFFSET_PTR => "ROW_BIND_OFFSET_PTR".to_string(),
        StatementAttribute::SQL_ATTR_ROW_BIND_TYPE => "ROW_BIND_TYPE".to_string(),
        StatementAttribute::SQL_ATTR_ROW_NUMBER => "ROW_NUMBER".to_string(),
        StatementAttribute::SQL_ATTR_ROW_OPERATION_PTR => "ROW_OPERATION_PTR".to_string(),
        StatementAttribute::SQL_ATTR_ROW_STATUS_PTR => "ROW_STATUS_PTR".to_string(),
        StatementAttribute::SQL_ATTR_ROWS_FETCHED_PTR => "ROWS_FETCHED_PTR".to_string(),
        StatementAttribute::SQL_ATTR_ROW_ARRAY_SIZE => "ROW_ARRAY_SIZE".to_string(),
        StatementAttribute::SQL_ATTR_SIMULATE_CURSOR => "SIMULATE_CURSOR".to_string(),
        StatementAttribute::SQL_ATTR_USE_BOOKMARKS => "USE_BOOKMARKS".to_string(),
        StatementAttribute::SQL_ATTR_ASYNC_STMT_EVENT => "ASYNC_STMT_EVENT".to_string(),
        StatementAttribute::SQL_ATTR_SAMPLE_SIZE => "SAMPLE_SIZE".to_string(),
        StatementAttribute::SQL_ATTR_DYNAMIC_COLUMNS => "DYNAMIC_COLUMNS".to_string(),
        StatementAttribute::SQL_ATTR_TYPE_EXCEPTION_BEHAVIOR => {
            "TYPE_EXCEPTION_BEHAVIOR".to_string()
        }
        StatementAttribute::SQL_ATTR_LENGTH_EXCEPTION_BEHAVIOR => {
            "LENGTH_EXCEPTION_BEHAVIOR".to_string()
        }
        StatementAttribute::SQL_ATTR_METADATA_ID => "METADATA_ID".to_string(),
    }
}

/// handle_sql_type returns the proper sql type for the application's current behavior. When the
/// driver's version is set to ODBC 2.x, the datetime types must be mapped to different SQL types
pub fn handle_sql_type(odbc_version: AttrOdbcVersion, sql_type: SqlDataType) -> SqlDataType {
    match odbc_version {
        AttrOdbcVersion::Odbc2 => match sql_type {
            // code for SQL_DATE from ODBC 2 is used as SQL_DATETIME in ODBC 3
            SqlDataType::SQL_TYPE_DATE => SqlDataType::SQL_DATETIME,
            // code for SQL_TIME from ODBC 2 is used as SQL_EXT_TIME_OR_INTERVAL in ODBC 3
            SqlDataType::SQL_TYPE_TIME => SqlDataType::SQL_TIME_OR_INTERVAL,
            // code for SQL_TIMESTAMP from ODBC 2 is used as SQL_EXT_TIMESTAMP in ODBC 3
            SqlDataType::SQL_TYPE_TIMESTAMP => SqlDataType::SQL_TIMESTAMP,
            v => v,
        },
        _ => sql_type,
    }
}

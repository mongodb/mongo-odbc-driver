#![allow(unused)]
/// ^ TODO: remove that when implementations are complete
use odbc_sys::{
    BulkOperation, CDataType, Char, CompletionType, ConnectionAttribute, Desc, DriverConnectOption,
    EnvironmentAttribute, FetchOrientation, HDbc, HDesc, HEnv, HStmt, HWnd, Handle, HandleType,
    InfoType, Integer, Len, Nullability, ParamType, Pointer, RetCode, SmallInt, SqlDataType,
    SqlReturn, StatementAttribute, ULen, USmallInt, WChar,
};

#[no_mangle]
pub extern "C" fn SQLAllocHandle(
    handle_type: HandleType,
    input_handle: Handle,
    output_handle: *mut Handle,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLBindCol(
    hstmt: HStmt,
    col_number: USmallInt,
    target_type: CDataType,
    target_value: Pointer,
    buffer_length: Len,
    length_or_indicatior: *mut Len,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLBindParameter(
    hstmt: HStmt,
    parameter_number: USmallInt,
    input_output_type: ParamType,
    value_type: CDataType,
    parmeter_type: SqlDataType,
    column_size: ULen,
    decimal_digits: SmallInt,
    parameter_value_ptr: Pointer,
    buffer_length: Len,
    str_len_or_ind_ptr: *mut Len,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLBrowseConnect(
    connection_handle: HDbc,
    in_connection_string: *const Char,
    string_length: SmallInt,
    out_connection_string: *mut Char,
    buffer_length: SmallInt,
    out_buffer_length: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLBrowseConnectW(
    connection_handle: HDbc,
    in_connection_string: *const WChar,
    string_length: SmallInt,
    out_connection_string: *mut WChar,
    buffer_length: SmallInt,
    out_buffer_length: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLBulkOperations(
    statement_handle: HStmt,
    operation: BulkOperation,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLCancel(statement_handle: HStmt) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLCancelHandle(handle_type: HandleType, handle: Handle) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLCloseCursor(statement_handle: HStmt) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLColAttribute(
    statement_handle: HStmt,
    column_number: USmallInt,
    field_identifier: Desc,
    character_attribute_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
    numeric_attribute_ptr: *mut Len,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLColAttributeW(
    statement_handle: HStmt,
    column_number: USmallInt,
    field_identifier: Desc,
    character_attribute_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
    numeric_attribute_ptr: *mut Len,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLColumnPrivileges(
    statement_handle: HStmt,
    catalog_name: *const Char,
    catalog_name_length: SmallInt,
    schema_name: *const Char,
    schema_name_length: SmallInt,
    table_name: *const Char,
    table_name_length: SmallInt,
    column_name: *const Char,
    column_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLColumnPrivilegesW(
    statement_handle: HStmt,
    catalog_name: *const WChar,
    catalog_name_length: SmallInt,
    schema_name: *const WChar,
    schema_name_length: SmallInt,
    table_name: *const WChar,
    table_name_length: SmallInt,
    column_name: *const WChar,
    column_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLColumns(
    statement_handle: HStmt,
    catalog_name: *const Char,
    catalog_name_length: SmallInt,
    schema_name: *const Char,
    schema_name_length: SmallInt,
    table_name: *const Char,
    table_name_length: SmallInt,
    column_name: *const Char,
    column_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLColumnsW(
    statement_handle: HStmt,
    catalog_name: *const WChar,
    catalog_name_length: SmallInt,
    schema_name: *const WChar,
    schema_name_length: SmallInt,
    table_name: *const WChar,
    table_name_length: SmallInt,
    column_name: *const WChar,
    column_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLCompleteAsync(
    handle_type: HandleType,
    handle: Handle,
    async_ret_code_ptr: *mut RetCode,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLConnect(
    connection_handle: HDbc,
    server_name: *const Char,
    name_length_1: SmallInt,
    user_name: *const Char,
    name_length_2: SmallInt,
    authentication: *const Char,
    name_length_3: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLConnectW(
    connection_handle: HDbc,
    server_name: *const WChar,
    name_length_1: SmallInt,
    user_name: *const WChar,
    name_length_2: SmallInt,
    authentication: *const WChar,
    name_length_3: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLCopyDesc(source_desc_handle: HDesc, target_desc_handle: HDesc) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLDataSources(
    environment_handle: HEnv,
    direction: FetchOrientation,
    server_name: *mut Char,
    buffer_length_1: SmallInt,
    name_length_1: *mut SmallInt,
    description: *mut Char,
    buffer_length_2: SmallInt,
    name_length_2: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLDataSourcesW(
    environment_handle: HEnv,
    direction: FetchOrientation,
    server_name: *mut WChar,
    buffer_length_1: SmallInt,
    name_length_1: *mut SmallInt,
    description: *mut WChar,
    buffer_length_2: SmallInt,
    name_length_2: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLDescribeCol(
    hstmt: HStmt,
    col_number: USmallInt,
    col_name: *mut Char,
    buffer_length: SmallInt,
    name_length: *mut SmallInt,
    data_type: *mut SqlDataType,
    col_size: *mut ULen,
    decimal_digits: *mut SmallInt,
    nullable: *mut Nullability,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLDescribeColW(
    hstmt: HStmt,
    col_number: USmallInt,
    col_name: *mut WChar,
    buffer_length: SmallInt,
    name_length: *mut SmallInt,
    data_type: *mut SqlDataType,
    col_size: *mut ULen,
    decimal_digits: *mut SmallInt,
    nullable: *mut Nullability,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLDescribeParam(
    statement_handle: HStmt,
    parameter_number: USmallInt,
    data_type_ptr: *mut SqlDataType,
    parameter_size_ptr: *mut ULen,
    decimal_digits_ptr: *mut SmallInt,
    nullable_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLDisconnect(connection_handle: HDbc) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLDriverConnect(
    connection_handle: HDbc,
    window_handle: HWnd,
    in_connection_string: *const Char,
    string_length_1: SmallInt,
    out_connection_string: *mut Char,
    buffer_length: SmallInt,
    string_length_2: *mut SmallInt,
    drive_completion: DriverConnectOption,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLDriverConnectW(
    connection_handle: HDbc,
    window_handle: HWnd,
    in_connection_string: *const WChar,
    string_length_1: SmallInt,
    out_connection_string: *mut WChar,
    buffer_length: SmallInt,
    string_length_2: *mut SmallInt,
    driver_completion: DriverConnectOption,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLDrivers(
    henv: HEnv,
    direction: FetchOrientation,
    driver_desc: *mut Char,
    driver_desc_max: SmallInt,
    out_driver_desc: *mut SmallInt,
    driver_attributes: *mut Char,
    drvr_attr_max: SmallInt,
    out_drvr_attr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLDriversW(
    henv: HEnv,
    direction: FetchOrientation,
    driver_desc: *mut WChar,
    driver_desc_max: SmallInt,
    out_driver_desc: *mut SmallInt,
    driver_attributes: *mut WChar,
    drvr_attr_max: SmallInt,
    out_drvr_attr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLEndTran(
    handle_type: HandleType,
    handle: Handle,
    completion_type: CompletionType,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLExecDirect(
    statement_handle: HStmt,
    statement_text: *const Char,
    text_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLExecDirectW(
    statement_handle: HStmt,
    statement_text: *const WChar,
    text_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLExecute(statement_handle: HStmt) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLFetch(statement_handle: HStmt) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLFetchScroll(
    statement_handle: HStmt,
    fetch_orientation: FetchOrientation,
    fetch_offset: Len,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLForeignKeys(
    statement_handle: HStmt,
    pk_catalog_name: *const Char,
    pk_catalog_name_length: SmallInt,
    pk_schema_name: *const Char,
    pk_schema_name_length: SmallInt,
    pk_table_name: *const Char,
    pk_table_name_length: SmallInt,
    fk_catalog_name: *const Char,
    fk_catalog_name_length: SmallInt,
    fk_schema_name: *const Char,
    fk_schema_name_length: SmallInt,
    fk_table_name: *const Char,
    fk_table_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLForeignKeysW(
    statement_handle: HStmt,
    pk_catalog_name: *const WChar,
    pk_catalog_name_length: SmallInt,
    pk_schema_name: *const WChar,
    pk_schema_name_length: SmallInt,
    pk_table_name: *const WChar,
    pk_table_name_length: SmallInt,
    fk_catalog_name: *const WChar,
    fk_catalog_name_length: SmallInt,
    fk_schema_name: *const WChar,
    fk_schema_name_length: SmallInt,
    fk_table_name: *const WChar,
    fk_table_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLFreeHandle(handle_type: HandleType, handle: Handle) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetConnectAttr(
    connection_handle: HDbc,
    attribute: ConnectionAttribute,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetConnectAttrW(
    connection_handle: HDbc,
    attribute: ConnectionAttribute,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetCursorName(
    statement_handle: HStmt,
    cursor_name: *mut Char,
    buffer_length: SmallInt,
    name_length_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetCursorNameW(
    statement_handle: HStmt,
    cursor_name: *mut WChar,
    buffer_length: SmallInt,
    name_length_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetData(
    statement_handle: HStmt,
    col_or_param_num: USmallInt,
    target_type: CDataType,
    target_value_ptr: Pointer,
    buffer_length: Len,
    str_len_or_ind_ptr: *mut Len,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetDescField(
    descriptor_handle: HDesc,
    record_number: SmallInt,
    field_identifier: SmallInt,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetDescFieldW(
    descriptor_handle: HDesc,
    record_number: SmallInt,
    field_identifier: SmallInt,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetDescRec(
    descriptor_handle: HDesc,
    record_number: SmallInt,
    name: *mut Char,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
    type_ptr: *mut SmallInt,
    sub_type_ptr: *mut SmallInt,
    length_ptr: *mut Len,
    precision_ptr: *mut SmallInt,
    scale_ptr: *mut SmallInt,
    nullable_ptr: *mut Nullability,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetDescRecW(
    descriptor_handle: HDesc,
    record_number: SmallInt,
    name: *mut WChar,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
    type_ptr: *mut SmallInt,
    sub_type_ptr: *mut SmallInt,
    length_ptr: *mut Len,
    precision_ptr: *mut SmallInt,
    scale_ptr: *mut SmallInt,
    nullable_ptr: *mut Nullability,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetDiagField(
    handle_type: HandleType,
    handle: Handle,
    record_rumber: SmallInt,
    diag_identifier: SmallInt,
    diag_info_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetDiagFieldW(
    handle_type: HandleType,
    handle: Handle,
    record_rumber: SmallInt,
    diag_identifier: SmallInt,
    diag_info_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetDiagRec(
    handle_type: HandleType,
    handle: Handle,
    rec_number: SmallInt,
    state: *mut Char,
    native_error_ptr: *mut Integer,
    message_text: *mut Char,
    buffer_length: SmallInt,
    text_length_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetDiagRecW(
    handle_type: HandleType,
    handle: Handle,
    record_rumber: SmallInt,
    state: *mut WChar,
    native_error_ptr: *mut Integer,
    message_text: *mut WChar,
    buffer_length: SmallInt,
    text_length_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetEnvAttr(
    environment_handle: HEnv,
    attribute: EnvironmentAttribute,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length: *mut Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetEnvAttrW(
    environment_handle: HEnv,
    attribute: EnvironmentAttribute,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length: *mut Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetInfo(
    connection_handle: HDbc,
    info_type: InfoType,
    info_value_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetInfoW(
    connection_handle: HDbc,
    info_type: InfoType,
    info_value_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetStmtAttr(
    handle: HStmt,
    attribute: StatementAttribute,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetStmtAttrW(
    handle: HStmt,
    attribute: StatementAttribute,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLGetTypeInfo(handle: HStmt, data_type: SqlDataType) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLMoreResults(handle: HStmt) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLNativeSql(
    connection_handle: HDbc,
    in_statement_text: *const Char,
    in_statement_len: Integer,
    out_statement_text: *mut Char,
    buffer_len: Integer,
    out_statement_len: *mut Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLNativeSqlW(
    connection_handle: HDbc,
    in_statement_text: *const WChar,
    in_statement_len: Integer,
    out_statement_text: *mut WChar,
    buffer_len: Integer,
    out_statement_len: *mut Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLNumParams(
    statement_handle: HStmt,
    param_count_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLNumResultCols(
    statement_handle: HStmt,
    column_count_ptr: *mut SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLParamData(hstmt: HStmt, value_ptr_ptr: *mut Pointer) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLPrepare(
    hstmt: HStmt,
    statement_text: *const Char,
    text_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLPrepareW(
    hstmt: HStmt,
    statement_text: *const WChar,
    text_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLPrimaryKeys(
    statement_handle: HStmt,
    catalog_name: *const Char,
    catalog_name_length: SmallInt,
    schema_name: *const Char,
    schema_name_length: SmallInt,
    table_name: *const Char,
    table_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLPrimaryKeysW(
    statement_handle: HStmt,
    catalog_name: *const WChar,
    catalog_name_length: SmallInt,
    schema_name: *const WChar,
    schema_name_length: SmallInt,
    table_name: *const WChar,
    table_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLProcedureColumns(
    statement_handle: HStmt,
    catalog_name: *const Char,
    catalog_name_length: SmallInt,
    schema_name: *const Char,
    schema_name_length: SmallInt,
    proc_name: *const Char,
    proc_name_length: SmallInt,
    column_name: *const Char,
    column_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLProcedureColumnsW(
    statement_handle: HStmt,
    catalog_name: *const WChar,
    catalog_name_length: SmallInt,
    schema_name: *const WChar,
    schema_name_length: SmallInt,
    proc_name: *const WChar,
    proc_name_length: SmallInt,
    column_name: *const WChar,
    column_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLProcedures(
    statement_handle: HStmt,
    catalog_name: *const Char,
    catalog_name_length: SmallInt,
    schema_name: *const Char,
    schema_name_length: SmallInt,
    proc_name: *const Char,
    proc_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLProceduresW(
    statement_handle: HStmt,
    catalog_name: *const WChar,
    catalog_name_length: SmallInt,
    schema_name: *const WChar,
    schema_name_length: SmallInt,
    proc_name: *const WChar,
    proc_name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLPutData(
    statement_handle: HStmt,
    data_ptr: Pointer,
    str_len_or_ind_ptr: Len,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLRowCount(statement_handle: HStmt, row_count_ptr: *mut Len) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSetConnectAttr(
    hdbc: HDbc,
    attr: ConnectionAttribute,
    value: Pointer,
    str_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSetConnectAttrW(
    hdbc: HDbc,
    attr: ConnectionAttribute,
    value: Pointer,
    str_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSetCursorName(
    statement_handle: HStmt,
    cursor_name: *const Char,
    name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSetCursorNameW(
    statement_handle: HStmt,
    cursor_name: *const WChar,
    name_length: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSetDescField(
    desc_handle: HDesc,
    rec_number: SmallInt,
    field_identifier: SmallInt,
    value_ptr: Pointer,
    buffer_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSetDescRec(
    desc_handle: HDesc,
    rec_number: SmallInt,
    desc_type: SmallInt,
    desc_sub_type: SmallInt,
    length: Len,
    precision: SmallInt,
    scale: SmallInt,
    data_ptr: Pointer,
    string_length_ptr: *const Len,
    indicator_ptr: *const Len,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSetEnvAttr(
    environment_handle: HEnv,
    attribute: EnvironmentAttribute,
    value: Pointer,
    string_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSetEnvAttrW(
    environment_handle: HEnv,
    attribute: EnvironmentAttribute,
    value: Pointer,
    string_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSetStmtAttr(
    hstmt: HStmt,
    attr: StatementAttribute,
    value: Pointer,
    str_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSetStmtAttrW(
    hstmt: HStmt,
    attr: StatementAttribute,
    value: Pointer,
    str_length: Integer,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSpecialColumns(
    statement_handle: HStmt,
    identifier_type: SmallInt,
    catalog_name: *const Char,
    catalog_name_length: SmallInt,
    schema_name: *const Char,
    schema_name_length: SmallInt,
    table_name: *const Char,
    table_name_length: SmallInt,
    scope: SmallInt,
    nullable: Nullability,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLSpecialColumnsW(
    statement_handle: HStmt,
    identifier_type: SmallInt,
    catalog_name: *const WChar,
    catalog_name_length: SmallInt,
    schema_name: *const WChar,
    schema_name_length: SmallInt,
    table_name: *const WChar,
    table_name_length: SmallInt,
    scope: SmallInt,
    nullable: Nullability,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLStatistics(
    statement_handle: HStmt,
    catalog_name: *const Char,
    catalog_name_length: SmallInt,
    schema_name: *const Char,
    schema_name_length: SmallInt,
    table_name: *const Char,
    table_name_length: SmallInt,
    unique: SmallInt,
    reserved: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLTablePrivileges(
    statement_handle: HStmt,
    catalog_name: *const Char,
    name_length_1: SmallInt,
    schema_name: *const Char,
    name_length_2: SmallInt,
    table_name: *const Char,
    name_length_3: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLTablesPrivilegesW(
    statement_handle: HStmt,
    catalog_name: *const WChar,
    name_length_1: SmallInt,
    schema_name: *const WChar,
    name_length_2: SmallInt,
    table_name: *const WChar,
    name_length_3: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLTables(
    statement_handle: HStmt,
    catalog_name: *const Char,
    name_length_1: SmallInt,
    schema_name: *const Char,
    name_length_2: SmallInt,
    table_name: *const Char,
    name_length_3: SmallInt,
    table_type: *const Char,
    name_length_4: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

#[no_mangle]
pub extern "C" fn SQLTablesW(
    statement_handle: HStmt,
    catalog_name: *const WChar,
    name_length_1: SmallInt,
    schema_name: *const WChar,
    name_length_2: SmallInt,
    table_name: *const WChar,
    name_length_3: SmallInt,
    table_type: *const WChar,
    name_length_4: SmallInt,
) -> SqlReturn {
    SqlReturn::ERROR
}

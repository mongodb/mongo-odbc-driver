use crate::{
    api::util::unsupported_function,
    handles::{
        Connection, ConnectionState, Env, EnvState, MongoHandle, MongoHandleRef, Statement,
        StatementState,
    },
};
use odbc_sys::{
    BulkOperation, CDataType, Char, CompletionType, ConnectionAttribute, Desc, DriverConnectOption,
    EnvironmentAttribute, FetchOrientation, HDbc, HDesc, HEnv, HStmt, HWnd, Handle, HandleType,
    InfoType, Integer, Len, Nullability, ParamType, Pointer, RetCode, SmallInt, SqlDataType,
    SqlReturn, StatementAttribute, ULen, USmallInt, WChar,
};
use std::sync::RwLock;

#[no_mangle]
pub extern "C" fn SQLAllocHandle(
    handle_type: HandleType,
    input_handle: Handle,
    output_handle: *mut Handle,
) -> SqlReturn {
    match sql_alloc_handle(handle_type, input_handle as *mut _, output_handle) {
        Ok(_) => SqlReturn::SUCCESS,
        Err(_) => SqlReturn::INVALID_HANDLE,
    }
}

fn sql_alloc_handle(
    handle_type: HandleType,
    input_handle: *mut MongoHandle,
    output_handle: *mut Handle,
) -> Result<(), ()> {
    match handle_type {
        HandleType::Env => {
            let env = RwLock::new(Env::with_state(EnvState::Allocated));
            let mh = Box::new(MongoHandle::Env(env));
            unsafe {
                *output_handle = Box::into_raw(mh) as *mut _;
            }
            Ok(())
        }
        HandleType::Dbc => {
            // input handle cannot be NULL
            if input_handle.is_null() {
                return Err(());
            }
            // input handle must be an Env
            let env = unsafe { (*input_handle).as_env().ok_or(())? };
            let conn = RwLock::new(Connection::with_state(
                input_handle,
                ConnectionState::Allocated,
            ));
            let mut env_contents = (*env).write().unwrap();
            let mh = Box::new(MongoHandle::Connection(conn));
            let mh_ptr = Box::into_raw(mh);
            env_contents.connections.insert(mh_ptr);
            env_contents.state = EnvState::ConnectionAllocated;
            unsafe { *output_handle = mh_ptr as *mut _ }
            Ok(())
        }
        HandleType::Stmt => {
            // input handle cannot be NULL
            if input_handle.is_null() {
                return Err(());
            }
            // input handle must be an Connection
            let conn = unsafe { (*input_handle).as_connection().ok_or(())? };
            let stmt = RwLock::new(Statement::with_state(
                input_handle,
                StatementState::Allocated,
            ));
            let mut conn_contents = (*conn).write().unwrap();
            let mh = Box::new(MongoHandle::Statement(stmt));
            let mh_ptr = Box::into_raw(mh);
            conn_contents.statements.insert(mh_ptr);
            conn_contents.state = ConnectionState::StatementAllocated;
            unsafe { *output_handle = mh_ptr as *mut _ }
            Ok(())
        }
        HandleType::Desc => {
            unimplemented!();
        }
    }
}

#[no_mangle]
pub extern "C" fn SQLBindCol(
    _hstmt: HStmt,
    _col_number: USmallInt,
    _target_type: CDataType,
    _target_value: Pointer,
    _buffer_length: Len,
    _length_or_indicatior: *mut Len,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLBindParameter(
    hstmt: HStmt,
    _parameter_number: USmallInt,
    _input_output_type: ParamType,
    _value_type: CDataType,
    _parmeter_type: SqlDataType,
    _column_size: ULen,
    _decimal_digits: SmallInt,
    _parameter_value_ptr: Pointer,
    _buffer_length: Len,
    _str_len_or_ind_ptr: *mut Len,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hstmt), "SQLBindParameter")
}

#[no_mangle]
pub extern "C" fn SQLBrowseConnect(
    connection_handle: HDbc,
    _in_connection_string: *const Char,
    _string_length: SmallInt,
    _out_connection_string: *mut Char,
    _buffer_length: SmallInt,
    _out_buffer_length: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLBrowseConnect")
}

#[no_mangle]
pub extern "C" fn SQLBrowseConnectW(
    _connection_handle: HDbc,
    _in_connection_string: *const WChar,
    _string_length: SmallInt,
    _out_connection_string: *mut WChar,
    _buffer_length: SmallInt,
    _out_buffer_length: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLBulkOperations(
    statement_handle: HStmt,
    _operation: BulkOperation,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLBulkOperations")
}

#[no_mangle]
pub extern "C" fn SQLCancel(_statement_handle: HStmt) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLCancelHandle(_handle_type: HandleType, _handle: Handle) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLCloseCursor(_statement_handle: HStmt) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLColAttribute(
    statement_handle: HStmt,
    _column_number: USmallInt,
    _field_identifier: Desc,
    _character_attribute_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
    _numeric_attribute_ptr: *mut Len,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLColAttribute")
}

#[no_mangle]
pub extern "C" fn SQLColAttributeW(
    _statement_handle: HStmt,
    _column_number: USmallInt,
    _field_identifier: Desc,
    _character_attribute_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
    _numeric_attribute_ptr: *mut Len,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLColumnPrivileges(
    statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _table_name: *const Char,
    _table_name_length: SmallInt,
    _column_name: *const Char,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unsupported_function(
        MongoHandleRef::from(statement_handle),
        "SQLColumnPrivileges",
    )
}

#[no_mangle]
pub extern "C" fn SQLColumnPrivilegesW(
    _statement_handle: HStmt,
    _catalog_name: *const WChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WChar,
    _schema_name_length: SmallInt,
    _table_name: *const WChar,
    _table_name_length: SmallInt,
    _column_name: *const WChar,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLColumns(
    statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _table_name: *const Char,
    _table_name_length: SmallInt,
    _column_name: *const Char,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLColumns")
}

#[no_mangle]
pub extern "C" fn SQLColumnsW(
    _statement_handle: HStmt,
    _catalog_name: *const WChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WChar,
    _schema_name_length: SmallInt,
    _table_name: *const WChar,
    _table_name_length: SmallInt,
    _column_name: *const WChar,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLCompleteAsync(
    _handle_type: HandleType,
    handle: Handle,
    _async_ret_code_ptr: *mut RetCode,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(handle), "SQLCompleteAsync")
}

#[no_mangle]
pub extern "C" fn SQLConnect(
    connection_handle: HDbc,
    _server_name: *const Char,
    _name_length_1: SmallInt,
    _user_name: *const Char,
    _name_length_2: SmallInt,
    _authentication: *const Char,
    _name_length_3: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLConnect")
}

#[no_mangle]
pub extern "C" fn SQLConnectW(
    connection_handle: HDbc,
    _server_name: *const WChar,
    _name_length_1: SmallInt,
    _user_name: *const WChar,
    _name_length_2: SmallInt,
    _authentication: *const WChar,
    _name_length_3: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLConnectW")
}

#[no_mangle]
pub extern "C" fn SQLCopyDesc(_source_desc_handle: HDesc, _target_desc_handle: HDesc) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLDataSources(
    environment_handle: HEnv,
    _direction: FetchOrientation,
    _server_name: *mut Char,
    _buffer_length_1: SmallInt,
    _name_length_1: *mut SmallInt,
    _description: *mut Char,
    _buffer_length_2: SmallInt,
    _name_length_2: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(environment_handle), "SQLDataSources")
}

#[no_mangle]
pub extern "C" fn SQLDataSourcesW(
    environment_handle: HEnv,
    _direction: FetchOrientation,
    _server_name: *mut WChar,
    _buffer_length_1: SmallInt,
    _name_length_1: *mut SmallInt,
    _description: *mut WChar,
    _buffer_length_2: SmallInt,
    _name_length_2: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(environment_handle), "SQLDataSourcesW")
}

#[no_mangle]
pub extern "C" fn SQLDescribeCol(
    hstmt: HStmt,
    _col_number: USmallInt,
    _col_name: *mut Char,
    _buffer_length: SmallInt,
    _name_length: *mut SmallInt,
    _data_type: *mut SqlDataType,
    _col_size: *mut ULen,
    _decimal_digits: *mut SmallInt,
    _nullable: *mut Nullability,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hstmt), "SQLDescribeCol")
}

#[no_mangle]
pub extern "C" fn SQLDescribeColW(
    _hstmt: HStmt,
    _col_number: USmallInt,
    _col_name: *mut WChar,
    _buffer_length: SmallInt,
    _name_length: *mut SmallInt,
    _data_type: *mut SqlDataType,
    _col_size: *mut ULen,
    _decimal_digits: *mut SmallInt,
    _nullable: *mut Nullability,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLDescribeParam(
    statement_handle: HStmt,
    _parameter_number: USmallInt,
    _data_type_ptr: *mut SqlDataType,
    _parameter_size_ptr: *mut ULen,
    _decimal_digits_ptr: *mut SmallInt,
    _nullable_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLDescribeParam")
}

#[no_mangle]
pub extern "C" fn SQLDisconnect(_connection_handle: HDbc) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLDriverConnect(
    connection_handle: HDbc,
    _window_handle: HWnd,
    _in_connection_string: *const Char,
    _string_length_1: SmallInt,
    _out_connection_string: *mut Char,
    _buffer_length: SmallInt,
    _string_length_2: *mut SmallInt,
    _drive_completion: DriverConnectOption,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLDriverConnect")
}

#[no_mangle]
pub extern "C" fn SQLDriverConnectW(
    _connection_handle: HDbc,
    _window_handle: HWnd,
    _in_connection_string: *const WChar,
    _string_length_1: SmallInt,
    _out_connection_string: *mut WChar,
    _buffer_length: SmallInt,
    _string_length_2: *mut SmallInt,
    _driver_completion: DriverConnectOption,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLDrivers(
    henv: HEnv,
    _direction: FetchOrientation,
    _driver_desc: *mut Char,
    _driver_desc_max: SmallInt,
    _out_driver_desc: *mut SmallInt,
    _driver_attributes: *mut Char,
    _drvr_attr_max: SmallInt,
    _out_drvr_attr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(henv), "SQLDrivers")
}

#[no_mangle]
pub extern "C" fn SQLDriversW(
    henv: HEnv,
    _direction: FetchOrientation,
    _driver_desc: *mut WChar,
    _driver_desc_max: SmallInt,
    _out_driver_desc: *mut SmallInt,
    _driver_attributes: *mut WChar,
    _drvr_attr_max: SmallInt,
    _out_drvr_attr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(henv), "SQLDriversW")
}

#[no_mangle]
pub extern "C" fn SQLEndTran(
    _handle_type: HandleType,
    _handle: Handle,
    _completion_type: CompletionType,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLExecDirect(
    statement_handle: HStmt,
    _statement_text: *const Char,
    _text_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLExecDirect")
}

#[no_mangle]
pub extern "C" fn SQLExecDirectW(
    statement_handle: HStmt,
    _statement_text: *const WChar,
    _text_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLExecDirectW")
}

#[no_mangle]
pub extern "C" fn SQLExecute(statement_handle: HStmt) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLExecute")
}

#[no_mangle]
pub extern "C" fn SQLFetch(_statement_handle: HStmt) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLFetchScroll(
    _statement_handle: HStmt,
    _fetch_orientation: FetchOrientation,
    _fetch_offset: Len,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLForeignKeys(
    statement_handle: HStmt,
    _pk_catalog_name: *const Char,
    _pk_catalog_name_length: SmallInt,
    _pk_schema_name: *const Char,
    _pk_schema_name_length: SmallInt,
    _pk_table_name: *const Char,
    _pk_table_name_length: SmallInt,
    _fk_catalog_name: *const Char,
    _fk_catalog_name_length: SmallInt,
    _fk_schema_name: *const Char,
    _fk_schema_name_length: SmallInt,
    _fk_table_name: *const Char,
    _fk_table_name_length: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLForeignKeys")
}

#[no_mangle]
pub extern "C" fn SQLForeignKeysW(
    _statement_handle: HStmt,
    _pk_catalog_name: *const WChar,
    _pk_catalog_name_length: SmallInt,
    _pk_schema_name: *const WChar,
    _pk_schema_name_length: SmallInt,
    _pk_table_name: *const WChar,
    _pk_table_name_length: SmallInt,
    _fk_catalog_name: *const WChar,
    _fk_catalog_name_length: SmallInt,
    _fk_schema_name: *const WChar,
    _fk_schema_name_length: SmallInt,
    _fk_table_name: *const WChar,
    _fk_table_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLFreeHandle(handle_type: HandleType, handle: Handle) -> SqlReturn {
    match sql_free_handle(handle_type, handle as *mut _) {
        Ok(_) => SqlReturn::SUCCESS,
        Err(_) => SqlReturn::INVALID_HANDLE,
    }
}

fn sql_free_handle(handle_type: HandleType, handle: *mut MongoHandle) -> Result<(), ()> {
    match handle_type {
        // By making Boxes to the types and letting them go out of
        // scope, they will be dropped.
        HandleType::Env => {
            let _ = unsafe { (*handle).as_env().ok_or(())? };
        }
        HandleType::Dbc => {
            let conn = unsafe { (*handle).as_connection().ok_or(())? };
            let mut env_contents = unsafe {
                (*conn.write().unwrap().env)
                    .as_env()
                    .ok_or(())?
                    .write()
                    .unwrap()
            };
            env_contents.connections.remove(&handle);
            if env_contents.connections.is_empty() {
                env_contents.state = EnvState::Allocated;
            }
        }
        HandleType::Stmt => {
            let stmt = unsafe { (*handle).as_statement().ok_or(())? };
            // Actually reading this value would make ASAN fail, but this
            // is what the ODBC standard expects.
            let mut conn_contents = unsafe {
                (*stmt.write().unwrap().connection)
                    .as_connection()
                    .ok_or(())?
                    .write()
                    .unwrap()
            };
            conn_contents.statements.remove(&handle);
            if conn_contents.statements.is_empty() {
                conn_contents.state = ConnectionState::Connected;
            }
        }
        HandleType::Desc => {
            unimplemented!();
        }
    }
    // create the Box at the end to ensure Drop only occurs when there are no errors due
    // to incorrect handle type.
    let _ = unsafe { Box::from_raw(handle) };
    Ok(())
}

#[no_mangle]
pub extern "C" fn SQLFreeStmt(_statement_handle: HStmt, _option: SmallInt) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetConnectAttr(
    connection_handle: HDbc,
    _attribute: ConnectionAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLGetConnectAttr")
}

#[no_mangle]
pub extern "C" fn SQLGetConnectAttrW(
    _connection_handle: HDbc,
    _attribute: ConnectionAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetCursorName(
    statement_handle: HStmt,
    _cursor_name: *mut Char,
    _buffer_length: SmallInt,
    _name_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLGetCursorName")
}

#[no_mangle]
pub extern "C" fn SQLGetCursorNameW(
    _statement_handle: HStmt,
    _cursor_name: *mut WChar,
    _buffer_length: SmallInt,
    _name_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetData(
    _statement_handle: HStmt,
    _col_or_param_num: USmallInt,
    _target_type: CDataType,
    _target_value_ptr: Pointer,
    _buffer_length: Len,
    _str_len_or_ind_ptr: *mut Len,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetDescField(
    _descriptor_handle: HDesc,
    _record_number: SmallInt,
    _field_identifier: SmallInt,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetDescFieldW(
    _descriptor_handle: HDesc,
    _record_number: SmallInt,
    _field_identifier: SmallInt,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetDescRec(
    _descriptor_handle: HDesc,
    _record_number: SmallInt,
    _name: *mut Char,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
    _type_ptr: *mut SmallInt,
    _sub_type_ptr: *mut SmallInt,
    _length_ptr: *mut Len,
    _precision_ptr: *mut SmallInt,
    _scale_ptr: *mut SmallInt,
    _nullable_ptr: *mut Nullability,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetDescRecW(
    _descriptor_handle: HDesc,
    _record_number: SmallInt,
    _name: *mut WChar,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
    _type_ptr: *mut SmallInt,
    _sub_type_ptr: *mut SmallInt,
    _length_ptr: *mut Len,
    _precision_ptr: *mut SmallInt,
    _scale_ptr: *mut SmallInt,
    _nullable_ptr: *mut Nullability,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetDiagField(
    _handle_type: HandleType,
    handle: Handle,
    _record_rumber: SmallInt,
    _diag_identifier: SmallInt,
    _diag_info_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(handle), "SQLGetDiagField")
}

#[no_mangle]
pub extern "C" fn SQLGetDiagFieldW(
    _handle_type: HandleType,
    _handle: Handle,
    _record_rumber: SmallInt,
    _diag_identifier: SmallInt,
    _diag_info_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetDiagRec(
    _handle_type: HandleType,
    handle: Handle,
    _rec_number: SmallInt,
    _state: *mut Char,
    _native_error_ptr: *mut Integer,
    _message_text: *mut Char,
    _buffer_length: SmallInt,
    _text_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(handle), "SQLGetDiagRec")
}

#[no_mangle]
pub extern "C" fn SQLGetDiagRecW(
    handle_type: HandleType,
    handle: Handle,
    rec_number: SmallInt,
    state: *mut WChar,
    native_error_ptr: *mut Integer,
    message_text: *mut WChar,
    buffer_length: SmallInt,
    text_length_ptr: *mut SmallInt,
) -> SqlReturn {
    if rec_number < 1 || buffer_length < 0 {
        return SqlReturn::ERROR;
    }
    let mongo_handle = handle as *mut MongoHandle;
    // Make the record number zero-indexed
    let rec_number = (rec_number - 1) as usize;
    match handle_type {
        HandleType::Env => match unsafe { (*mongo_handle).as_env() } {
            Some(env) => {
                let env_contents = (*env).read().unwrap();
                match env_contents.errors.get(rec_number) {
                    Some(odbc_err) => util::get_diag_rec(
                        odbc_err,
                        state,
                        message_text,
                        buffer_length,
                        text_length_ptr,
                        native_error_ptr,
                    ),
                    None => SqlReturn::NO_DATA,
                }
            }
            None => SqlReturn::INVALID_HANDLE,
        },
        HandleType::Dbc => match unsafe { (*mongo_handle).as_connection() } {
            Some(dbc) => {
                let dbc_contents = (*dbc).read().unwrap();
                match dbc_contents.errors.get(rec_number) {
                    Some(odbc_err) => util::get_diag_rec(
                        odbc_err,
                        state,
                        message_text,
                        buffer_length,
                        text_length_ptr,
                        native_error_ptr,
                    ),
                    None => SqlReturn::NO_DATA,
                }
            }
            None => SqlReturn::INVALID_HANDLE,
        },
        HandleType::Stmt => match unsafe { (*mongo_handle).as_statement() } {
            Some(stmt) => {
                let stmt_contents = (*stmt).read().unwrap();
                match stmt_contents.errors.get(rec_number) {
                    Some(odbc_err) => util::get_diag_rec(
                        odbc_err,
                        state,
                        message_text,
                        buffer_length,
                        text_length_ptr,
                        native_error_ptr,
                    ),
                    None => SqlReturn::NO_DATA,
                }
            }
            None => SqlReturn::INVALID_HANDLE,
        },
        HandleType::Desc => unimplemented!(),
    }
}

#[no_mangle]
pub extern "C" fn SQLGetEnvAttr(
    environment_handle: HEnv,
    _attribute: EnvironmentAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length: *mut Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(environment_handle), "SQLGetEnvAttr")
}

#[no_mangle]
pub extern "C" fn SQLGetEnvAttrW(
    _environment_handle: HEnv,
    _attribute: EnvironmentAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length: *mut Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetInfo(
    connection_handle: HDbc,
    _info_type: InfoType,
    _info_value_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLGetInfo")
}

#[no_mangle]
pub extern "C" fn SQLGetInfoW(
    _connection_handle: HDbc,
    _info_type: InfoType,
    _info_value_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetStmtAttr(
    handle: HStmt,
    _attribute: StatementAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(handle), "SQLGetStmtAttr")
}

#[no_mangle]
pub extern "C" fn SQLGetStmtAttrW(
    _handle: HStmt,
    _attribute: StatementAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetTypeInfo(_handle: HStmt, _data_type: SqlDataType) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLMoreResults(_handle: HStmt) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLNativeSql(
    connection_handle: HDbc,
    _in_statement_text: *const Char,
    _in_statement_len: Integer,
    _out_statement_text: *mut Char,
    _buffer_len: Integer,
    _out_statement_len: *mut Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLNativeSql")
}

#[no_mangle]
pub extern "C" fn SQLNativeSqlW(
    _connection_handle: HDbc,
    _in_statement_text: *const WChar,
    _in_statement_len: Integer,
    _out_statement_text: *mut WChar,
    _buffer_len: Integer,
    _out_statement_len: *mut Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLNumParams(
    statement_handle: HStmt,
    _param_count_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLNumParams")
}

#[no_mangle]
pub extern "C" fn SQLNumResultCols(
    statement_handle: HStmt,
    _column_count_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLNumResultCols")
}

#[no_mangle]
pub extern "C" fn SQLParamData(hstmt: HStmt, _value_ptr_ptr: *mut Pointer) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hstmt), "SQLParamData")
}

#[no_mangle]
pub extern "C" fn SQLPrepare(
    hstmt: HStmt,
    _statement_text: *const Char,
    _text_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hstmt), "SQLPrepare")
}

#[no_mangle]
pub extern "C" fn SQLPrepareW(
    hstmt: HStmt,
    _statement_text: *const WChar,
    _text_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hstmt), "SQLPrepareW")
}

#[no_mangle]
pub extern "C" fn SQLPrimaryKeys(
    statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _table_name: *const Char,
    _table_name_length: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLPrimaryKeys")
}

#[no_mangle]
pub extern "C" fn SQLPrimaryKeysW(
    _statement_handle: HStmt,
    _catalog_name: *const WChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WChar,
    _schema_name_length: SmallInt,
    _table_name: *const WChar,
    _table_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLProcedureColumns(
    statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _proc_name: *const Char,
    _proc_name_length: SmallInt,
    _column_name: *const Char,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unsupported_function(
        MongoHandleRef::from(statement_handle),
        "SQLProcedureColumns",
    )
}

#[no_mangle]
pub extern "C" fn SQLProcedureColumnsW(
    statement_handle: HStmt,
    _catalog_name: *const WChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WChar,
    _schema_name_length: SmallInt,
    _proc_name: *const WChar,
    _proc_name_length: SmallInt,
    _column_name: *const WChar,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unsupported_function(
        MongoHandleRef::from(statement_handle),
        "SQLProcedureColumnsW",
    )
}

#[no_mangle]
pub extern "C" fn SQLProcedures(
    statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _proc_name: *const Char,
    _proc_name_length: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLProcedures")
}

#[no_mangle]
pub extern "C" fn SQLProceduresW(
    statement_handle: HStmt,
    _catalog_name: *const WChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WChar,
    _schema_name_length: SmallInt,
    _proc_name: *const WChar,
    _proc_name_length: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLProceduresW")
}

#[no_mangle]
pub extern "C" fn SQLPutData(
    statement_handle: HStmt,
    _data_ptr: Pointer,
    _str_len_or_ind_ptr: Len,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLPutData")
}

#[no_mangle]
pub extern "C" fn SQLRowCount(_statement_handle: HStmt, _row_count_ptr: *mut Len) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetConnectAttr(
    hdbc: HDbc,
    _attr: ConnectionAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hdbc), "SQLSetConnectAttr")
}

#[no_mangle]
pub extern "C" fn SQLSetConnectAttrW(
    _hdbc: HDbc,
    _attr: ConnectionAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetCursorName(
    statement_handle: HStmt,
    _cursor_name: *const Char,
    _name_length: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLSetCursorName")
}

#[no_mangle]
pub extern "C" fn SQLSetCursorNameW(
    _statement_handle: HStmt,
    _cursor_name: *const WChar,
    _name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetDescField(
    _desc_handle: HDesc,
    _rec_number: SmallInt,
    _field_identifier: SmallInt,
    _value_ptr: Pointer,
    _buffer_length: Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetDescRec(
    _desc_handle: HDesc,
    _rec_number: SmallInt,
    _desc_type: SmallInt,
    _desc_sub_type: SmallInt,
    _length: Len,
    _precision: SmallInt,
    _scale: SmallInt,
    _data_ptr: Pointer,
    _string_length_ptr: *const Len,
    _indicator_ptr: *const Len,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetPos(
    statement_handle: HStmt,
    _row_number: ULen,
    _operation: USmallInt,
    _lock_type: USmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLSetPos")
}

#[no_mangle]
pub extern "C" fn SQLSetEnvAttr(
    environment_handle: HEnv,
    _attribute: EnvironmentAttribute,
    _value: Pointer,
    _string_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(environment_handle), "SQLSetEnvAttr")
}

#[no_mangle]
pub extern "C" fn SQLSetEnvAttrW(
    _environment_handle: HEnv,
    _attribute: EnvironmentAttribute,
    _value: Pointer,
    _string_length: Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetStmtAttr(
    hstmt: HStmt,
    _attr: StatementAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hstmt), "SQLSetStmtAttr")
}

#[no_mangle]
pub extern "C" fn SQLSetStmtAttrW(
    _hstmt: HStmt,
    _attr: StatementAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSpecialColumns(
    statement_handle: HStmt,
    _identifier_type: SmallInt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _table_name: *const Char,
    _table_name_length: SmallInt,
    _scope: SmallInt,
    _nullable: Nullability,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLSpecialColumns")
}

#[no_mangle]
pub extern "C" fn SQLSpecialColumnsW(
    _statement_handle: HStmt,
    _identifier_type: SmallInt,
    _catalog_name: *const WChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WChar,
    _schema_name_length: SmallInt,
    _table_name: *const WChar,
    _table_name_length: SmallInt,
    _scope: SmallInt,
    _nullable: Nullability,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLStatistics(
    _statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _table_name: *const Char,
    _table_name_length: SmallInt,
    _unique: SmallInt,
    _reserved: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLTablePrivileges(
    statement_handle: HStmt,
    _catalog_name: *const Char,
    _name_length_1: SmallInt,
    _schema_name: *const Char,
    _name_length_2: SmallInt,
    _table_name: *const Char,
    _name_length_3: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLTablePrivileges")
}

#[no_mangle]
pub extern "C" fn SQLTablesPrivilegesW(
    _statement_handle: HStmt,
    _catalog_name: *const WChar,
    _name_length_1: SmallInt,
    _schema_name: *const WChar,
    _name_length_2: SmallInt,
    _table_name: *const WChar,
    _name_length_3: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLTables(
    statement_handle: HStmt,
    _catalog_name: *const Char,
    _name_length_1: SmallInt,
    _schema_name: *const Char,
    _name_length_2: SmallInt,
    _table_name: *const Char,
    _name_length_3: SmallInt,
    _table_type: *const Char,
    _name_length_4: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLTables")
}

#[no_mangle]
pub extern "C" fn SQLTablesW(
    _statement_handle: HStmt,
    _catalog_name: *const WChar,
    _name_length_1: SmallInt,
    _schema_name: *const WChar,
    _name_length_2: SmallInt,
    _table_name: *const WChar,
    _name_length_3: SmallInt,
    _table_type: *const WChar,
    _name_length_4: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

mod util {
    use crate::{errors::ODBCError, handles::MongoHandle};
    use odbc_sys::{Integer, SmallInt, SqlReturn, WChar};
    use std::{cmp::min, ptr::copy_nonoverlapping};

    /// set_sql_state writes the given sql state to the [`output_ptr`].
    pub fn set_sql_state(sql_state: &str, output_ptr: *mut WChar) {
        let sql_state = &format!("{}\0", sql_state);
        let state_u16 = sql_state.encode_utf16().collect::<Vec<u16>>();
        unsafe {
            copy_nonoverlapping(state_u16.as_ptr(), output_ptr, 6);
        }
    }

    /// set_error_message writes [`error_message`] to the [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the error message should be truncated
    /// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    pub fn set_error_message(
        error_message: String,
        output_ptr: *mut WChar,
        buffer_len: usize,
        text_length_ptr: *mut SmallInt,
    ) -> SqlReturn {
        unsafe {
            // Check if the entire error message plus a null terminator can fit in the buffer;
            // we should truncate the error message if it's too long.
            let mut message_u16 = error_message.encode_utf16().collect::<Vec<u16>>();
            let message_len = message_u16.len();
            let num_chars = min(message_len + 1, buffer_len);
            message_u16.resize(num_chars - 1, 0);
            message_u16.push('\u{0}' as u16);
            copy_nonoverlapping(message_u16.as_ptr(), output_ptr, num_chars);
            // Store the number of characters in the error message string, excluding the
            // null terminator, in text_length_ptr
            *text_length_ptr = (num_chars - 1) as SmallInt;
            if num_chars < message_len {
                SqlReturn::SUCCESS_WITH_INFO
            } else {
                SqlReturn::SUCCESS
            }
        }
    }

    /// get_diag_rec copies the given ODBC error's diagnostic information
    /// into the provided pointers.
    pub fn get_diag_rec(
        error: &ODBCError,
        state: *mut WChar,
        message_text: *mut WChar,
        buffer_length: SmallInt,
        text_length_ptr: *mut SmallInt,
        native_error_ptr: *mut Integer,
    ) -> SqlReturn {
        unsafe { *native_error_ptr = error.get_native_err_code() };
        set_sql_state(error.get_sql_state(), state);
        set_error_message(
            error.get_error_message(),
            message_text,
            buffer_length as usize,
            text_length_ptr,
        )
    }

    pub fn unsupported_function(handle: &mut MongoHandle, name: &'static str) -> SqlReturn {
        handle.clear_diagnostics();
        handle.add_diag_info(ODBCError::Unimplemented(name));
        SqlReturn::ERROR
    }
}

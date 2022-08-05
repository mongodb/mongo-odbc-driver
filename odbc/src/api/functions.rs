use crate::{
    api::{
        definitions::*,
        errors::ODBCError,
        functions::util::{input_wtext_to_string, set_str_length, unsupported_function},
    },
    handles::definitions::*,
};
use num_traits::FromPrimitive;
use odbc_sys::{
    BulkOperation, CDataType, Char, CompletionType, ConnectionAttribute, Desc, DriverConnectOption,
    EnvironmentAttribute, FetchOrientation, HDbc, HDesc, HEnv, HStmt, HWnd, Handle, HandleType,
    InfoType, Integer, Len, Nullability, ParamType, Pointer, RetCode, SmallInt, SqlDataType,
    SqlReturn, StatementAttribute, ULen, USmallInt, WChar,
};
use std::{env, mem::size_of, sync::RwLock};

#[no_mangle]
pub extern "C" fn SQLAllocHandle(
    handle_type: HandleType,
    input_handle: Handle,
    output_handle: *mut Handle,
) -> SqlReturn {
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLBulkOperations(
    statement_handle: HStmt,
    _operation: BulkOperation,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLBulkOperations")
}

#[no_mangle]
pub extern "C" fn SQLCancel(_statement_handle: HStmt) -> SqlReturn {
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLCancelHandle(_handle_type: HandleType, _handle: Handle) -> SqlReturn {
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLCloseCursor(_statement_handle: HStmt) -> SqlReturn {
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLCompleteAsync(
    _handle_type: HandleType,
    handle: Handle,
    _async_ret_code_ptr: *mut RetCode,
) -> SqlReturn {
    dbg!();
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
    dbg!();
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
    dbg!();
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLConnectW")
}

#[no_mangle]
pub extern "C" fn SQLCopyDesc(_source_desc_handle: HDesc, _target_desc_handle: HDesc) -> SqlReturn {
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLDescribeParam")
}

#[no_mangle]
pub extern "C" fn SQLDisconnect(connection_handle: HDbc) -> SqlReturn {
    dbg!();
    let conn_handle = MongoHandleRef::from(connection_handle);
    let conn = (*conn_handle).as_connection();
    if conn.is_none() {
        conn_handle.add_diag_info(ODBCError::InvalidHandleType("Connection"));
        return SqlReturn::ERROR;
    }
    // set the mongo_connection to None. This will cause the previous mongo_connection
    // to drop and disconnect.
    conn.unwrap().write().unwrap().mongo_connection = None;
    SqlReturn::SUCCESS
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
    dbg!();
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLDriverConnect")
}

#[no_mangle]
pub extern "C" fn SQLDriverConnectW(
    connection_handle: HDbc,
    _window_handle: HWnd,
    in_connection_string: *const WChar,
    string_length_1: SmallInt,
    _out_connection_string: *mut WChar,
    _buffer_length: SmallInt,
    _string_length_2: *mut SmallInt,
    _driver_completion: DriverConnectOption,
) -> SqlReturn {
    dbg!();
    dbg!("!!!!");
    let conn_handle = MongoHandleRef::from(connection_handle);
    let conn = (*conn_handle).as_connection();
    if conn.is_none() {
        conn_handle.add_diag_info(ODBCError::InvalidHandleType("Connection"));
        return SqlReturn::ERROR;
    }
    let conn = conn.unwrap();
    let uri = input_wtext_to_string(in_connection_string, string_length_1 as usize);
    let database = env::var("SQL_ATTR_CURRENT_CATALOG").ok();
    let connect_result = mongo_odbc_core::MongoConnection::connect(
        &uri,     // uri
        database, // current_db
        None,     // op timeout i32
        None,     // login timeout i32
    );
    match connect_result {
        Ok(mc) => {
            let mut conn_writer = conn.write().unwrap();
            conn_writer.attributes.current_db = mc.current_db.clone();
            conn_writer.mongo_connection = Some(mc);
            SqlReturn::SUCCESS
        }
        Err(error) => {
            match error {
                mongo_odbc_core::Error::MongoDriver(mdbe) => {
                    conn_handle.add_diag_info(ODBCError::MongoError(mdbe));
                }
                mongo_odbc_core::Error::UriFormatError(s) => {
                    conn_handle.add_diag_info(ODBCError::UriFormatError(s));
                }
            };
            SqlReturn::ERROR
        }
    }
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
    dbg!();
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
    dbg!();
    unsupported_function(MongoHandleRef::from(henv), "SQLDriversW")
}

#[no_mangle]
pub extern "C" fn SQLEndTran(
    _handle_type: HandleType,
    _handle: Handle,
    _completion_type: CompletionType,
) -> SqlReturn {
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLExecDirect(
    statement_handle: HStmt,
    _statement_text: *const Char,
    _text_length: Integer,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLExecDirect")
}

#[no_mangle]
pub extern "C" fn SQLExecDirectW(
    statement_handle: HStmt,
    _statement_text: *const WChar,
    _text_length: Integer,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLExecDirectW")
}

#[no_mangle]
pub extern "C" fn SQLExecute(statement_handle: HStmt) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLExecute")
}

#[no_mangle]
pub extern "C" fn SQLFetch(_statement_handle: HStmt) -> SqlReturn {
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLFetchScroll(
    _statement_handle: HStmt,
    _fetch_orientation: FetchOrientation,
    _fetch_offset: Len,
) -> SqlReturn {
    dbg!();
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
    dbg!();
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
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLFreeHandle(handle_type: HandleType, handle: Handle) -> SqlReturn {
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetCursorName(
    statement_handle: HStmt,
    _cursor_name: *mut Char,
    _buffer_length: SmallInt,
    _name_length_ptr: *mut SmallInt,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLGetCursorName")
}

#[no_mangle]
pub extern "C" fn SQLGetCursorNameW(
    _statement_handle: HStmt,
    _cursor_name: *mut WChar,
    _buffer_length: SmallInt,
    _name_length_ptr: *mut SmallInt,
) -> SqlReturn {
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
    // TODO: implement me, this is stubbed for test
    SqlReturn::SUCCESS
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
    dbg!();
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
    dbg!();
    if rec_number < 1 || buffer_length < 0 {
        dbg!();
        return SqlReturn::ERROR;
    }
    let mongo_handle = handle as *mut MongoHandle;
    dbg!();
    // Make the record number zero-indexed
    let rec_number = (rec_number - 1) as usize;
    dbg!(
        handle_type,
        rec_number,
        state,
        native_error_ptr,
        message_text,
        buffer_length,
        text_length_ptr
    );
    match handle_type {
        HandleType::Env => match unsafe { (*mongo_handle).as_env() } {
            Some(env) => {
                dbg!();
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
                dbg!();
                let dbc_contents = (*dbc).read().unwrap();
                dbg!(&dbc_contents.errors, rec_number);
                dbg!(&dbc_contents.errors.get(rec_number));
                match dbc_contents.errors.get(rec_number) {
                    Some(odbc_err) => {
                        dbg!(odbc_err);
                        util::get_diag_rec(
                            odbc_err,
                            state,
                            message_text,
                            buffer_length,
                            text_length_ptr,
                            native_error_ptr,
                        )
                    }
                    None => {
                        dbg!("NO DATA");
                        SqlReturn::NO_DATA
                    }
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
    dbg!();
    unsupported_function(MongoHandleRef::from(environment_handle), "SQLGetEnvAttr")
}

#[no_mangle]
pub extern "C" fn SQLGetEnvAttrW(
    environment_handle: HEnv,
    attribute: EnvironmentAttribute,
    value_ptr: Pointer,
    _buffer_length: Integer,
    string_length: *mut Integer,
) -> SqlReturn {
    dbg!();
    let env_handle = MongoHandleRef::from(environment_handle);
    env_handle.clear_diagnostics();
    match env_handle.as_env() {
        None => SqlReturn::INVALID_HANDLE,
        Some(env) => {
            let env_contents = env.read().unwrap();
            if value_ptr.is_null() {
                set_str_length(string_length, 0);
            } else {
                set_str_length(string_length, size_of::<Integer>() as Integer);
                match attribute {
                    EnvironmentAttribute::OdbcVersion => unsafe {
                        *(value_ptr as *mut OdbcVersion) = env_contents.attributes.odbc_ver;
                    },
                    EnvironmentAttribute::OutputNts => unsafe {
                        *(value_ptr as *mut SqlBool) = env_contents.attributes.output_nts;
                    },
                    EnvironmentAttribute::ConnectionPooling => unsafe {
                        *(value_ptr as *mut ConnectionPooling) =
                            env_contents.attributes.connection_pooling;
                    },
                    EnvironmentAttribute::CpMatch => unsafe {
                        *(value_ptr as *mut CpMatch) = env_contents.attributes.cp_match;
                    },
                }
            }
            SqlReturn::SUCCESS
        }
    }
}

#[no_mangle]
pub extern "C" fn SQLGetInfo(
    connection_handle: HDbc,
    _info_type: InfoType,
    _info_value_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLGetInfo")
}

#[no_mangle]
pub extern "C" fn SQLGetInfoW(
    connection_handle: HDbc,
    _info_type: InfoType,
    _info_value_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLGetInfoW")
}

#[no_mangle]
pub extern "C" fn SQLGetStmtAttr(
    handle: HStmt,
    _attribute: StatementAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(handle), "SQLGetStmtAttr")
}

#[no_mangle]
pub extern "C" fn SQLGetStmtAttrW(
    handle: HStmt,
    attribute: StatementAttribute,
    value_ptr: Pointer,
    _buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    dbg!();
    let stmt_handle = MongoHandleRef::from(handle);
    stmt_handle.clear_diagnostics();
    match stmt_handle.as_statement() {
        None => SqlReturn::INVALID_HANDLE,
        Some(stmt) => {
            let stmt_contents = stmt.read().unwrap();
            if !value_ptr.is_null() {
                // Most attributes have type SQLULEN, so default to the size of that
                // type.
                set_str_length(string_length_ptr, size_of::<ULen>() as Integer);
                match attribute {
                    StatementAttribute::AppRowDesc => unsafe {
                        *(value_ptr as *mut Pointer) = stmt_contents.attributes.app_row_desc;
                        set_str_length(string_length_ptr, size_of::<Pointer>() as Integer);
                    },
                    StatementAttribute::AppParamDesc => unsafe {
                        *(value_ptr as *mut Pointer) = stmt_contents.attributes.app_param_desc;
                        set_str_length(string_length_ptr, size_of::<Pointer>() as Integer);
                    },
                    StatementAttribute::ImpRowDesc => unsafe {
                        *(value_ptr as *mut Pointer) = stmt_contents.attributes.imp_row_desc;
                        set_str_length(string_length_ptr, size_of::<Pointer>() as Integer);
                    },
                    StatementAttribute::ImpParamDesc => unsafe {
                        *(value_ptr as *mut Pointer) = stmt_contents.attributes.imp_param_desc;
                        set_str_length(string_length_ptr, size_of::<Pointer>() as Integer);
                    },
                    StatementAttribute::FetchBookmarkPtr => unsafe {
                        *(value_ptr as *mut _) = stmt_contents.attributes.fetch_bookmark_ptr;
                        set_str_length(string_length_ptr, size_of::<*mut Len>() as Integer);
                    },
                    StatementAttribute::CursorScrollable => unsafe {
                        *(value_ptr as *mut CursorScrollable) =
                            stmt_contents.attributes.cursor_scrollable;
                    },
                    StatementAttribute::CursorSensitivity => unsafe {
                        *(value_ptr as *mut CursorSensitivity) =
                            stmt_contents.attributes.cursor_sensitivity;
                    },
                    StatementAttribute::AsyncEnable => unsafe {
                        *(value_ptr as *mut AsyncEnable) = stmt_contents.attributes.async_enable;
                    },
                    StatementAttribute::Concurrency => unsafe {
                        *(value_ptr as *mut Concurrency) = stmt_contents.attributes.concurrency;
                    },
                    StatementAttribute::CursorType => unsafe {
                        *(value_ptr as *mut CursorType) = stmt_contents.attributes.cursor_type;
                    },
                    StatementAttribute::EnableAutoIpd => unsafe {
                        *(value_ptr as *mut SqlBool) = stmt_contents.attributes.enable_auto_ipd;
                    },
                    StatementAttribute::KeysetSize => unsafe {
                        *(value_ptr as *mut ULen) = 0;
                    },
                    StatementAttribute::MaxLength => unsafe {
                        *(value_ptr as *mut ULen) = stmt_contents.attributes.max_length;
                    },
                    StatementAttribute::MaxRows => unsafe {
                        *(value_ptr as *mut ULen) = stmt_contents.attributes.max_rows;
                    },
                    StatementAttribute::NoScan => unsafe {
                        *(value_ptr as *mut NoScan) = stmt_contents.attributes.no_scan;
                    },
                    StatementAttribute::ParamBindOffsetPtr => unsafe {
                        *(value_ptr as *mut _) = stmt_contents.attributes.param_bind_offset_ptr;
                        set_str_length(string_length_ptr, size_of::<*mut ULen>() as Integer)
                    },
                    StatementAttribute::ParamBindType => unsafe {
                        *(value_ptr as *mut ULen) = stmt_contents.attributes.param_bind_type;
                    },
                    StatementAttribute::ParamOpterationPtr => unsafe {
                        *(value_ptr as *mut _) = stmt_contents.attributes.param_operation_ptr;
                        set_str_length(string_length_ptr, size_of::<*mut USmallInt>() as Integer)
                    },
                    StatementAttribute::ParamStatusPtr => unsafe {
                        *(value_ptr as *mut _) = stmt_contents.attributes.param_status_ptr;
                        set_str_length(string_length_ptr, size_of::<*mut USmallInt>() as Integer)
                    },
                    StatementAttribute::ParamsProcessedPtr => unsafe {
                        *(value_ptr as *mut _) = stmt_contents.attributes.param_processed_ptr;
                        set_str_length(string_length_ptr, size_of::<*mut ULen>() as Integer)
                    },
                    StatementAttribute::ParamsetSize => unsafe {
                        *(value_ptr as *mut ULen) = stmt_contents.attributes.paramset_size;
                    },
                    StatementAttribute::QueryTimeout => unsafe {
                        *(value_ptr as *mut ULen) = stmt_contents.attributes.query_timeout;
                    },
                    StatementAttribute::RetrieveData => unsafe {
                        *(value_ptr as *mut RetrieveData) = stmt_contents.attributes.retrieve_data;
                    },
                    StatementAttribute::RowBindOffsetPtr => unsafe {
                        *(value_ptr as *mut _) = stmt_contents.attributes.row_bind_offset_ptr;
                        set_str_length(string_length_ptr, size_of::<*mut ULen>() as Integer)
                    },
                    StatementAttribute::RowBindType => unsafe {
                        *(value_ptr as *mut ULen) = stmt_contents.attributes.row_bind_type;
                    },
                    StatementAttribute::RowNumber => unsafe {
                        *(value_ptr as *mut ULen) = stmt_contents.attributes.row_number;
                    },
                    StatementAttribute::RowOperationPtr => unsafe {
                        *(value_ptr as *mut _) = stmt_contents.attributes.row_operation_ptr;
                        set_str_length(string_length_ptr, size_of::<*mut USmallInt>() as Integer)
                    },
                    StatementAttribute::RowStatusPtr => unsafe {
                        *(value_ptr as *mut _) = stmt_contents.attributes.row_status_ptr;
                        set_str_length(string_length_ptr, size_of::<*mut USmallInt>() as Integer)
                    },
                    StatementAttribute::RowsFetchedPtr => unsafe {
                        *(value_ptr as *mut _) = stmt_contents.attributes.rows_fetched_ptr;
                        set_str_length(string_length_ptr, size_of::<*mut ULen>() as Integer)
                    },
                    StatementAttribute::RowArraySize => unsafe {
                        *(value_ptr as *mut ULen) = stmt_contents.attributes.row_array_size;
                    },
                    StatementAttribute::SimulateCursor => unsafe {
                        *(value_ptr as *mut ULen) = stmt_contents.attributes.simulate_cursor;
                    },
                    StatementAttribute::UseBookmarks => unsafe {
                        *(value_ptr as *mut UseBookmarks) = stmt_contents.attributes.use_bookmarks;
                    },
                    StatementAttribute::AsyncStmtEvent => unsafe {
                        *(value_ptr as *mut _) = stmt_contents.attributes.async_stmt_event;
                    },
                    StatementAttribute::MetadataId => {
                        todo!();
                    }
                }
            }
            SqlReturn::SUCCESS
        }
    }
}

#[no_mangle]
pub extern "C" fn SQLGetTypeInfo(_handle: HStmt, _data_type: SqlDataType) -> SqlReturn {
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLMoreResults(_handle: HStmt) -> SqlReturn {
    dbg!();
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
    dbg!();
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
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLNumParams(
    statement_handle: HStmt,
    _param_count_ptr: *mut SmallInt,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLNumParams")
}

#[no_mangle]
pub extern "C" fn SQLNumResultCols(
    statement_handle: HStmt,
    _column_count_ptr: *mut SmallInt,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLNumResultCols")
}

#[no_mangle]
pub extern "C" fn SQLParamData(hstmt: HStmt, _value_ptr_ptr: *mut Pointer) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(hstmt), "SQLParamData")
}

#[no_mangle]
pub extern "C" fn SQLPrepare(
    hstmt: HStmt,
    _statement_text: *const Char,
    _text_length: Integer,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(hstmt), "SQLPrepare")
}

#[no_mangle]
pub extern "C" fn SQLPrepareW(
    hstmt: HStmt,
    _statement_text: *const WChar,
    _text_length: Integer,
) -> SqlReturn {
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLProceduresW")
}

#[no_mangle]
pub extern "C" fn SQLPutData(
    statement_handle: HStmt,
    _data_ptr: Pointer,
    _str_len_or_ind_ptr: Len,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLPutData")
}

#[no_mangle]
pub extern "C" fn SQLRowCount(_statement_handle: HStmt, _row_count_ptr: *mut Len) -> SqlReturn {
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetConnectAttr(
    hdbc: HDbc,
    _attr: ConnectionAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(hdbc), "SQLSetConnectAttr")
}

#[no_mangle]
pub extern "C" fn SQLSetConnectAttrW(
    _hdbc: HDbc,
    _attr: ConnectionAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetCursorName(
    statement_handle: HStmt,
    _cursor_name: *const Char,
    _name_length: SmallInt,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLSetCursorName")
}

#[no_mangle]
pub extern "C" fn SQLSetCursorNameW(
    _statement_handle: HStmt,
    _cursor_name: *const WChar,
    _name_length: SmallInt,
) -> SqlReturn {
    dbg!();
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
    dbg!();
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
    dbg!();
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetPos(
    statement_handle: HStmt,
    _row_number: ULen,
    _operation: USmallInt,
    _lock_type: USmallInt,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLSetPos")
}

#[no_mangle]
pub extern "C" fn SQLSetEnvAttr(
    environment_handle: HEnv,
    attribute: EnvironmentAttribute,
    value: Pointer,
    _string_length: Integer,
) -> SqlReturn {
    dbg!();
    //unsupported_function(MongoHandleRef::from(environment_handle), "SQLSetEnvAttr")
    SQLSetEnvAttrW(environment_handle, attribute, value, _string_length)
}

#[no_mangle]
pub extern "C" fn SQLSetEnvAttrW(
    environment_handle: HEnv,
    attribute: EnvironmentAttribute,
    value: Pointer,
    _string_length: Integer,
) -> SqlReturn {
    dbg!();
    let env_handle = MongoHandleRef::from(environment_handle);
    env_handle.clear_diagnostics();
    match env_handle.as_env() {
        None => SqlReturn::INVALID_HANDLE,
        Some(env) => match attribute {
            EnvironmentAttribute::OdbcVersion => match FromPrimitive::from_i32(value as i32) {
                Some(version) => {
                    let mut env_contents = (*env).write().unwrap();
                    env_contents.attributes.odbc_ver = version;
                    SqlReturn::SUCCESS
                }
                None => {
                    env_handle.add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_ODBC_VERSION"));
                    SqlReturn::ERROR
                }
            },
            EnvironmentAttribute::OutputNts => match FromPrimitive::from_i32(value as i32) {
                Some(SqlBool::True) => SqlReturn::SUCCESS,
                _ => {
                    env_handle.add_diag_info(ODBCError::Unimplemented("OUTPUT_NTS=SQL_FALSE"));
                    SqlReturn::ERROR
                }
            },
            EnvironmentAttribute::ConnectionPooling => {
                match FromPrimitive::from_i32(value as i32) {
                    Some(ConnectionPooling::Off) => SqlReturn::SUCCESS,
                    _ => {
                        env_handle.add_diag_info(ODBCError::OptionValueChanged(
                            "SQL_ATTR_CONNECTION_POOLING",
                            "SQL_CP_OFF",
                        ));
                        SqlReturn::SUCCESS_WITH_INFO
                    }
                }
            }
            EnvironmentAttribute::CpMatch => match FromPrimitive::from_i32(value as i32) {
                Some(CpMatch::Strict) => SqlReturn::SUCCESS,
                _ => {
                    env_handle.add_diag_info(ODBCError::OptionValueChanged(
                        "SQL_ATTR_CP_MATCH",
                        "SQL_CP_STRICT_MATCH",
                    ));
                    SqlReturn::SUCCESS_WITH_INFO
                }
            },
        },
    }
}

#[no_mangle]
pub extern "C" fn SQLSetStmtAttr(
    hstmt: HStmt,
    _attr: StatementAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    dbg!();
    unsupported_function(MongoHandleRef::from(hstmt), "SQLSetStmtAttr")
}

#[no_mangle]
pub extern "C" fn SQLSetStmtAttrW(
    hstmt: HStmt,
    attr: StatementAttribute,
    value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    dbg!();
    let stmt_handle = MongoHandleRef::from(hstmt);
    stmt_handle.clear_diagnostics();
    match stmt_handle.as_statement() {
        None => SqlReturn::INVALID_HANDLE,
        Some(stmt) => match attr {
            StatementAttribute::AppRowDesc => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_APP_ROW_DESC"));
                SqlReturn::ERROR
            }
            StatementAttribute::AppParamDesc => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_APP_PARAM_DESC"));
                SqlReturn::ERROR
            }
            StatementAttribute::ImpRowDesc => {
                // TODO: SQL_681, determine the correct SQL state
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_IMP_ROW_DESC"));
                SqlReturn::ERROR
            }
            StatementAttribute::ImpParamDesc => {
                // TODO: SQL_681, determine the correct SQL state
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_IMP_PARAM_DESC"));
                SqlReturn::ERROR
            }
            StatementAttribute::CursorScrollable => {
                match FromPrimitive::from_usize(value as usize) {
                    Some(CursorScrollable::NonScrollable) => SqlReturn::SUCCESS,
                    _ => {
                        stmt_handle.add_diag_info(ODBCError::InvalidAttrValue(
                            "SQL_ATTR_CURSOR_SCROLLABLE",
                        ));
                        SqlReturn::ERROR
                    }
                }
            }
            StatementAttribute::CursorSensitivity => match FromPrimitive::from_i32(value as i32) {
                Some(CursorSensitivity::Insensitive) => SqlReturn::SUCCESS,
                _ => {
                    stmt_handle
                        .add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_CURSOR_SENSITIVITY"));
                    SqlReturn::ERROR
                }
            },
            StatementAttribute::AsyncEnable => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_ASYNC_ENABLE"));
                SqlReturn::ERROR
            }
            StatementAttribute::Concurrency => match FromPrimitive::from_i32(value as i32) {
                Some(Concurrency::ReadOnly) => SqlReturn::SUCCESS,
                _ => {
                    stmt_handle.add_diag_info(ODBCError::OptionValueChanged(
                        "SQL_ATTR_CONCURRENCY",
                        "SQL_CONCUR_READ_ONLY",
                    ));
                    SqlReturn::SUCCESS_WITH_INFO
                }
            },
            StatementAttribute::CursorType => match FromPrimitive::from_i32(value as i32) {
                Some(CursorType::ForwardOnly) => SqlReturn::SUCCESS,
                _ => {
                    stmt_handle.add_diag_info(ODBCError::OptionValueChanged(
                        "SQL_ATTR_CURSOR_TYPE",
                        "SQL_CURSOR_FORWARD_ONLY",
                    ));
                    SqlReturn::SUCCESS_WITH_INFO
                }
            },
            StatementAttribute::EnableAutoIpd => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_ENABLE_AUTO_IPD"));
                SqlReturn::ERROR
            }
            StatementAttribute::FetchBookmarkPtr => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_FETCH_BOOKMARK_PTR"));
                SqlReturn::ERROR
            }
            StatementAttribute::KeysetSize => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_KEYSET_SIZE"));
                SqlReturn::ERROR
            }
            StatementAttribute::MaxLength => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_MAX_LENGTH"));
                SqlReturn::ERROR
            }
            StatementAttribute::MaxRows => {
                let mut stmt_contents = stmt.write().unwrap();
                stmt_contents.attributes.max_rows = value as ULen;
                SqlReturn::SUCCESS
            }
            StatementAttribute::NoScan => {
                match FromPrimitive::from_i32(value as i32) {
                    Some(ns) => {
                        let mut stmt_contents = stmt.write().unwrap();
                        stmt_contents.attributes.no_scan = ns
                    }
                    None => {
                        stmt_handle.add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_NOSCAN"))
                    }
                }
                SqlReturn::SUCCESS
            }
            StatementAttribute::ParamBindOffsetPtr => {
                stmt_handle
                    .add_diag_info(ODBCError::Unimplemented("SQL_ATTR_PARAM_BIND_OFFSET_PTR"));
                SqlReturn::ERROR
            }
            StatementAttribute::ParamBindType => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_PARAM_BIND_TYPE"));
                SqlReturn::ERROR
            }
            StatementAttribute::ParamOpterationPtr => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_PARAM_OPERATION_PTR"));
                SqlReturn::ERROR
            }
            StatementAttribute::ParamStatusPtr => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_PARAM_STATUS_PTR"));
                SqlReturn::ERROR
            }
            StatementAttribute::ParamsProcessedPtr => {
                stmt_handle
                    .add_diag_info(ODBCError::Unimplemented("SQL_ATTR_PARAMS_PROCESSED_PTR"));
                SqlReturn::ERROR
            }
            StatementAttribute::ParamsetSize => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_PARAMSET_SIZE"));
                SqlReturn::ERROR
            }
            StatementAttribute::QueryTimeout => {
                let mut stmt_contents = stmt.write().unwrap();
                stmt_contents.attributes.query_timeout = value as ULen;
                SqlReturn::SUCCESS
            }
            StatementAttribute::RetrieveData => match FromPrimitive::from_i32(value as i32) {
                Some(RetrieveData::Off) => SqlReturn::SUCCESS,
                _ => {
                    stmt_handle
                        .add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_RETRIEVE_DATA"));
                    SqlReturn::ERROR
                }
            },
            StatementAttribute::RowBindOffsetPtr => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_ROW_BIND_OFFSET_PTR"));
                SqlReturn::ERROR
            }
            StatementAttribute::RowBindType => {
                let mut stmt_contents = stmt.write().unwrap();
                stmt_contents.attributes.row_bind_type = value as ULen;
                SqlReturn::SUCCESS
            }
            StatementAttribute::RowNumber => {
                let mut stmt_contents = stmt.write().unwrap();
                stmt_contents.attributes.row_number = value as ULen;
                SqlReturn::SUCCESS
            }
            StatementAttribute::RowOperationPtr => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_ROW_OPERATION_PTR"));
                SqlReturn::ERROR
            }
            StatementAttribute::RowStatusPtr => {
                let mut stmt_contents = stmt.write().unwrap();
                stmt_contents.attributes.row_status_ptr = value as *mut USmallInt;
                SqlReturn::SUCCESS
            }
            StatementAttribute::RowsFetchedPtr => {
                let mut stmt_contents = stmt.write().unwrap();
                stmt_contents.attributes.rows_fetched_ptr = value as *mut ULen;
                SqlReturn::SUCCESS
            }
            StatementAttribute::RowArraySize => match FromPrimitive::from_i32(value as i32) {
                Some(ras) => {
                    let mut stmt_contents = stmt.write().unwrap();
                    stmt_contents.attributes.row_array_size = ras;
                    SqlReturn::SUCCESS
                }
                None => {
                    stmt_handle
                        .add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_ROW_ARRAY_SIZE"));
                    SqlReturn::ERROR
                }
            },
            StatementAttribute::SimulateCursor => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_SIMULATE_CURSOR"));
                SqlReturn::ERROR
            }
            StatementAttribute::UseBookmarks => match FromPrimitive::from_i32(value as i32) {
                Some(ub) => {
                    let mut stmt_contents = stmt.write().unwrap();
                    stmt_contents.attributes.use_bookmarks = ub;
                    SqlReturn::SUCCESS
                }
                None => {
                    stmt_handle
                        .add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_USE_BOOKMARKS"));
                    SqlReturn::ERROR
                }
            },
            StatementAttribute::AsyncStmtEvent => {
                stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_ASYNC_STMT_EVENT"));
                SqlReturn::ERROR
            }
            StatementAttribute::MetadataId => {
                todo!()
            }
        },
    }
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
    unimplemented!()
}

mod util {
    use crate::{api::errors::ODBCError, handles::definitions::MongoHandle};
    use odbc_sys::{Integer, SmallInt, SqlReturn, WChar};
    use std::{cmp::min, ptr::copy_nonoverlapping};

    /// input_wtext_to_string converts an input cstring to a rust String.
    /// It assumes nul termination if the supplied length is negative.
    #[allow(clippy::uninit_vec)]
    pub fn input_wtext_to_string(text: *const WChar, len: usize) -> String {
        if (len as isize) < 0 {
            let mut dst = Vec::new();
            let mut itr = text;
            unsafe {
                while *itr != 0 {
                    dst.push(*itr);
                    itr = itr.offset(1);
                }
            }
            return String::from_utf16_lossy(&dst);
        }

        let mut dst = Vec::with_capacity(len);
        unsafe {
            dst.set_len(len);
            copy_nonoverlapping(text, dst.as_mut_ptr(), len);
        }
        String::from_utf16_lossy(&dst)
    }

    /// set_sql_state writes the given sql state to the [`output_ptr`].
    pub fn set_sql_state(sql_state: &str, output_ptr: *mut WChar) {
        dbg!(sql_state);
        if output_ptr.is_null() {
            return;
        }
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
        dbg!(&error_message, output_ptr, buffer_len, text_length_ptr);
        assert!(!error_message.is_empty());
        unsafe {
            if output_ptr.is_null() {
                if !text_length_ptr.is_null() {
                    *text_length_ptr = 0 as SmallInt;
                }
                return SqlReturn::SUCCESS_WITH_INFO;
            }
            // Check if the entire error message plus a null terminator can fit in the buffer;
            // we should truncate the error message if it's too long.
            let mut message_u16 = error_message.encode_utf16().collect::<Vec<u16>>();
            let message_len = message_u16.len();
            let num_chars = min(message_len + 1, buffer_len);
            // It is possible that no buffer space has been allocated.
            if num_chars == 0 {
                return SqlReturn::SUCCESS_WITH_INFO;
            }
            message_u16.resize(num_chars - 1, 0);
            message_u16.push('\u{0}' as u16);
            copy_nonoverlapping(message_u16.as_ptr(), output_ptr, num_chars);
            // Store the number of characters in the error message string, excluding the
            // null terminator, in text_length_ptr
            if !text_length_ptr.is_null() {
                *text_length_ptr = (num_chars - 1) as SmallInt;
            }
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
        if !native_error_ptr.is_null() {
            unsafe { *native_error_ptr = error.get_native_err_code() };
        }
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

    /// set_str_length writes the given length to [`string_length_ptr`].
    pub fn set_str_length(string_length_ptr: *mut Integer, length: Integer) {
        if !string_length_ptr.is_null() {
            unsafe { *string_length_ptr = length }
        }
    }
}

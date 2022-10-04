use crate::api::functions::util::set_output_string;
use crate::{
    api::{
        definitions::*,
        errors::{ODBCError, Result},
        functions::util::{input_wtext_to_string, set_str_length, unsupported_function},
        odbc_uri::ODBCUri,
    },
    handles::definitions::*,
};
use mongo_odbc_core::{MongoColMetadata, MongoConnection, MongoDatabases, MongoStatement};
use num_traits::FromPrimitive;
use odbc_sys::{
    BulkOperation, CDataType, Char, CompletionType, ConnectionAttribute, Desc, DriverConnectOption,
    EnvironmentAttribute, FetchOrientation, HDbc, HDesc, HEnv, HStmt, HWnd, Handle, HandleType,
    InfoType, Integer, Len, Nullability, ParamType, Pointer, RetCode, SmallInt, SqlDataType,
    SqlReturn, StatementAttribute, ULen, USmallInt, WChar,
};
use std::{mem::size_of, sync::RwLock};

const NULL_HANDLE_ERROR: &str = "handle cannot be null";
const HANDLE_MUST_BE_ENV_ERROR: &str = "handle must be env";
const HANDLE_MUST_BE_CONN_ERROR: &str = "handle must be conn";
const HANDLE_MUST_BE_STMT_ERROR: &str = "handle must be stmt";

macro_rules! must_be_valid {
    ($maybe_handle:expr) => {{
        // force the expression
        let maybe_handle = $maybe_handle;
        if maybe_handle.is_none() {
            return SqlReturn::INVALID_HANDLE;
        }
        maybe_handle.unwrap()
    }};
}

macro_rules! unsafe_must_be_env {
    ($handle:expr) => {{
        let env = unsafe { (*$handle).as_env() };
        must_be_valid!(env)
    }};
}

macro_rules! unsafe_must_be_conn {
    ($handle:expr) => {{
        let conn = unsafe { (*$handle).as_connection() };
        must_be_valid!(conn)
    }};
}

macro_rules! unsafe_must_be_stmt {
    ($handle:expr) => {{
        let stmt = unsafe { (*$handle).as_statement() };
        must_be_valid!(stmt)
    }};
}

macro_rules! odbc_unwrap {
    ($value:expr, $handle:expr) => {{
        // force the expression
        let value = $value;
        if let Err(error) = value {
            $handle.add_diag_info(error.into());
            return SqlReturn::ERROR;
        }
        value.unwrap()
    }};
}

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
) -> Result<()> {
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
                return Err(ODBCError::InvalidHandleType(NULL_HANDLE_ERROR));
            }
            // input handle must be an Env
            let env = unsafe {
                (*input_handle)
                    .as_env()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_ENV_ERROR))?
            };
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
                return Err(ODBCError::InvalidHandleType(NULL_HANDLE_ERROR));
            }
            // input handle must be an Connection
            let conn = unsafe {
                (*input_handle)
                    .as_connection()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_CONN_ERROR))?
            };
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
    statement_handle: HStmt,
    column_number: USmallInt,
    field_identifier: Desc,
    character_attribute_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
    numeric_attribute_ptr: *mut Len,
) -> SqlReturn {
    let mongo_handle = MongoHandleRef::from(statement_handle);
    let stmt = must_be_valid!((*mongo_handle).as_statement());
    let string_col_attr = |f: &dyn Fn(&MongoColMetadata) -> &str| {
        let stmt_contents = stmt.read().unwrap();
        if stmt_contents.mongo_statement.is_none() {
            return set_output_string(
                "",
                character_attribute_ptr as *mut WChar,
                buffer_length as usize,
                string_length_ptr,
            );
        }
        let col_metadata = stmt_contents
            .mongo_statement
            .as_ref()
            .unwrap()
            .get_col_metadata(column_number);
        set_output_string(
            &(*f)(odbc_unwrap!(
                col_metadata,
                MongoHandleRef::from(statement_handle)
            )),
            character_attribute_ptr as *mut WChar,
            buffer_length as usize,
            string_length_ptr,
        )
    };
    let numeric_col_attr = |f: &dyn Fn(&MongoColMetadata) -> Len| {
        let stmt_contents = stmt.read().unwrap();
        if stmt_contents.mongo_statement.is_none() {
            unsafe {
                *numeric_attribute_ptr = 0 as Len;
            }
            return SqlReturn::SUCCESS;
        }
        let col_metadata = stmt_contents
            .mongo_statement
            .as_ref()
            .unwrap()
            .get_col_metadata(column_number);
        unsafe {
            *numeric_attribute_ptr = (*f)(odbc_unwrap!(
                col_metadata,
                MongoHandleRef::from(statement_handle)
            ));
        }
        return SqlReturn::SUCCESS;
    };
    match field_identifier {
        Desc::AutoUniqueValue => unsafe {
            *numeric_attribute_ptr = SqlBool::False as Len;
            return SqlReturn::SUCCESS;
        },
        Desc::Unnamed | Desc::Updatable => unsafe {
            *numeric_attribute_ptr = 0 as Len;
            return SqlReturn::SUCCESS;
        },
        Desc::Count => unsafe {
            let stmt_contents = stmt.read().unwrap();
            if stmt_contents.mongo_statement.is_none() {
                *numeric_attribute_ptr = 0 as Len;
                return SqlReturn::SUCCESS;
            }
            *numeric_attribute_ptr = stmt_contents
                .mongo_statement
                .as_ref()
                .unwrap()
                .get_resultset_metadata()
                .len() as Len;
            return SqlReturn::SUCCESS;
        },
        Desc::CaseSensitive => numeric_col_attr(&|x: &MongoColMetadata| {
            (if x.type_name == "string" {
                SqlBool::True
            } else {
                SqlBool::False
            }) as Len
        }),
        Desc::BaseColumnName => string_col_attr(&|x: &MongoColMetadata| x.base_col_name.as_ref()),
        Desc::BaseTableName => string_col_attr(&|x: &MongoColMetadata| x.base_table_name.as_ref()),
        Desc::CatalogName => string_col_attr(&|x: &MongoColMetadata| x.catalog_name.as_ref()),
        Desc::DisplaySize => {
            numeric_col_attr(&|x: &MongoColMetadata| x.display_size.unwrap_or(0) as Len)
        }
        Desc::FixedPrecScale => numeric_col_attr(&|x: &MongoColMetadata| x.fixed_prec_scale as Len),
        Desc::Label => string_col_attr(&|x: &MongoColMetadata| x.label.as_ref()),
        Desc::Length => numeric_col_attr(&|x: &MongoColMetadata| x.length.unwrap_or(0) as Len),
        Desc::LiteralPrefix | Desc::LiteralSuffix | Desc::LocalTypeName | Desc::SchemaName => {
            string_col_attr(&|_| "")
        }
        Desc::Name => string_col_attr(&|x: &MongoColMetadata| x.col_name.as_ref()),
        Desc::Nullable => numeric_col_attr(&|x: &MongoColMetadata| x.is_nullable as Len),
        Desc::OctetLength => {
            numeric_col_attr(&|x: &MongoColMetadata| x.octet_length.unwrap_or(0) as Len)
        }
        Desc::Precision => {
            numeric_col_attr(&|x: &MongoColMetadata| x.precision.unwrap_or(0) as Len)
        }
        Desc::Scale => numeric_col_attr(&|x: &MongoColMetadata| x.scale.unwrap_or(0) as Len),
        Desc::Searchable => numeric_col_attr(&|x: &MongoColMetadata| x.is_searchable as Len),
        Desc::TableName => string_col_attr(&|x: &MongoColMetadata| x.table_name.as_ref()),
        Desc::TypeName => string_col_attr(&|x: &MongoColMetadata| x.type_name.as_ref()),
        Desc::Type | Desc::ConciseType => {
            numeric_col_attr(&|x: &MongoColMetadata| x.sql_type.0 as Len)
        }
        Desc::Unsigned => numeric_col_attr(&|x: &MongoColMetadata| x.is_unsigned as Len),
        desc @ (Desc::OctetLengthPtr
        | Desc::DatetimeIntervalCode
        | Desc::IndicatorPtr
        | Desc::DataPtr
        | Desc::AllocType
        | Desc::ArraySize
        | Desc::ArrayStatusPtr
        | Desc::BindOffsetPtr
        | Desc::BindType
        | Desc::DatetimeIntervalPrecision
        | Desc::MaximumScale
        | Desc::MinimumScale
        | Desc::NumPrecRadix
        | Desc::ParameterType
        | Desc::RowsProcessedPtr
        | Desc::RowVer) => {
            mongo_handle
                .add_diag_info(ODBCError::UnsupportedFieldDescriptor(format!("{:?}", desc)));
            return SqlReturn::ERROR;
        }
    }
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
pub extern "C" fn SQLDisconnect(connection_handle: HDbc) -> SqlReturn {
    let conn_handle = MongoHandleRef::from(connection_handle);
    let conn = must_be_valid!((*conn_handle).as_connection());
    // set the mongo_connection to None. This will cause the previous mongo_connection
    // to drop and disconnect.
    conn.write().unwrap().mongo_connection = None;
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
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLDriverConnect")
}

fn sql_driver_connect(
    conn_handle: &RwLock<Connection>,
    odbc_uri_string: &str,
) -> Result<MongoConnection> {
    let conn_reader = conn_handle.read().unwrap();
    let mut odbc_uri = ODBCUri::new(odbc_uri_string)?;
    let mongo_uri = odbc_uri.remove_to_mongo_uri()?;
    let auth_src = odbc_uri.remove_or_else(|| "admin", &["auth_src"]);
    odbc_uri
        .remove(&["driver", "dsn"])
        .ok_or(ODBCError::MissingDriverOrDSNProperty)?;
    let database = if conn_reader.attributes.current_catalog.is_some() {
        conn_reader.attributes.current_catalog.as_deref()
    } else {
        odbc_uri.remove(&["database"])
    };
    let connection_timeout = conn_reader.attributes.connection_timeout;
    let login_timeout = conn_reader.attributes.login_timeout;
    let application_name = odbc_uri.remove(&["app_name", "application_name"]);
    // ODBCError has an impl From mongo_odbc_core::Error, but that does not
    // create an impl From Result<T, mongo_odbc_core::Error> to Result<T, ODBCError>
    // hence this bizarre Ok(func?) pattern.
    Ok(mongo_odbc_core::MongoConnection::connect(
        &mongo_uri,
        auth_src,
        database,
        connection_timeout,
        login_timeout,
        application_name,
    )?)
}

#[no_mangle]
pub extern "C" fn SQLDriverConnectW(
    connection_handle: HDbc,
    _window_handle: HWnd,
    in_connection_string: *const WChar,
    string_length_1: SmallInt,
    out_connection_string: *mut WChar,
    buffer_length: SmallInt,
    string_length_2: *mut SmallInt,
    driver_completion: DriverConnectOption,
) -> SqlReturn {
    let conn_handle = MongoHandleRef::from(connection_handle);
    // SQL_NO_PROMPT is the only option supported for DriverCompletion
    if driver_completion != DriverConnectOption::NoPrompt {
        conn_handle.add_diag_info(ODBCError::UnsupportedDriverConnectOption(format!(
            "{:?}",
            driver_completion
        )));
        return SqlReturn::ERROR;
    }
    let conn = must_be_valid!((*conn_handle).as_connection());
    let odbc_uri_string = input_wtext_to_string(in_connection_string, string_length_1 as usize);
    let mongo_connection = odbc_unwrap!(sql_driver_connect(conn, &odbc_uri_string), conn_handle);
    conn.write().unwrap().mongo_connection = Some(mongo_connection);
    let buffer_len = usize::try_from(buffer_length).unwrap();
    let sql_return = set_output_string(
        &odbc_uri_string,
        out_connection_string,
        buffer_len,
        string_length_2,
    );
    if sql_return == SqlReturn::SUCCESS_WITH_INFO {
        conn_handle.add_diag_info(ODBCError::OutStringTruncated(buffer_len));
    }
    sql_return
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

fn sql_free_handle(handle_type: HandleType, handle: *mut MongoHandle) -> Result<()> {
    match handle_type {
        // By making Boxes to the types and letting them go out of
        // scope, they will be dropped.
        HandleType::Env => {
            let _ = unsafe {
                (*handle)
                    .as_env()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_ENV_ERROR))?
            };
        }
        HandleType::Dbc => {
            let conn = unsafe {
                (*handle)
                    .as_connection()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_CONN_ERROR))?
            };
            let mut env_contents = unsafe {
                (*conn.write().unwrap().env)
                    .as_env()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_ENV_ERROR))?
                    .write()
                    .unwrap()
            };
            env_contents.connections.remove(&handle);
            if env_contents.connections.is_empty() {
                env_contents.state = EnvState::Allocated;
            }
        }
        HandleType::Stmt => {
            let stmt = unsafe {
                (*handle)
                    .as_statement()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_STMT_ERROR))?
            };
            // Actually reading this value would make ASAN fail, but this
            // is what the ODBC standard expects.
            let mut conn_contents = unsafe {
                (*stmt.write().unwrap().connection)
                    .as_connection()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_CONN_ERROR))?
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
    handle: Handle,
    _record_rumber: SmallInt,
    _diag_identifier: SmallInt,
    _diag_info_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(handle), "SQLGetDiagFieldW")
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

    let get_error = |errors: &Vec<ODBCError>| -> SqlReturn {
        match errors.get(rec_number) {
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
    };

    match handle_type {
        HandleType::Env => {
            let env = unsafe_must_be_env!(mongo_handle);
            get_error(&(*env).read().unwrap().errors.read().unwrap())
        }
        HandleType::Dbc => {
            let dbc = unsafe_must_be_conn!(mongo_handle);
            get_error(&(*dbc).read().unwrap().errors.read().unwrap())
        }
        HandleType::Stmt => {
            let stmt = unsafe_must_be_stmt!(mongo_handle);
            get_error(&(*stmt).read().unwrap().errors.read().unwrap())
        }
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
    environment_handle: HEnv,
    attribute: EnvironmentAttribute,
    value_ptr: Pointer,
    _buffer_length: Integer,
    string_length: *mut Integer,
) -> SqlReturn {
    let env_handle = MongoHandleRef::from(environment_handle);
    env_handle.clear_diagnostics();
    let env = must_be_valid!(env_handle.as_env());
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
                *(value_ptr as *mut ConnectionPooling) = env_contents.attributes.connection_pooling;
            },
            EnvironmentAttribute::CpMatch => unsafe {
                *(value_ptr as *mut CpMatch) = env_contents.attributes.cp_match;
            },
        }
    }
    SqlReturn::SUCCESS
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
    connection_handle: HDbc,
    _info_type: InfoType,
    _info_value_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
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
    let stmt_handle = MongoHandleRef::from(handle);
    stmt_handle.clear_diagnostics();
    let stmt = must_be_valid!(stmt_handle.as_statement());
    if value_ptr.is_null() {
        return SqlReturn::ERROR;
    }
    let stmt_contents = stmt.read().unwrap();
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
            *(value_ptr as *mut CursorScrollable) = stmt_contents.attributes.cursor_scrollable;
        },
        StatementAttribute::CursorSensitivity => unsafe {
            *(value_ptr as *mut CursorSensitivity) = stmt_contents.attributes.cursor_sensitivity;
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
    SqlReturn::SUCCESS
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
    column_count_ptr: *mut SmallInt,
) -> SqlReturn {
    let mongo_handle = MongoHandleRef::from(statement_handle);
    let stmt = must_be_valid!((*mongo_handle).as_statement());
    let stmt_contents = stmt.read().unwrap();
    let mongo_statement = stmt_contents.mongo_statement.as_ref();
    if mongo_statement.is_none() {
        unsafe {
            *column_count_ptr = 0;
        }
        return SqlReturn::SUCCESS;
    }
    unsafe {
        *column_count_ptr = mongo_statement.unwrap().get_resultset_metadata().len() as SmallInt;
    }
    SqlReturn::SUCCESS
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
pub extern "C" fn SQLRowCount(statement_handle: HStmt, row_count_ptr: *mut Len) -> SqlReturn {
    let mongo_handle = MongoHandleRef::from(statement_handle);
    // even though we always return 0, we must still assert that the proper handle
    // type is sent by the client.
    let _ = must_be_valid!((*mongo_handle).as_statement());
    unsafe {
        *row_count_ptr = 0 as Len;
    }
    SqlReturn::SUCCESS
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
    attribute: EnvironmentAttribute,
    value: Pointer,
    _string_length: Integer,
) -> SqlReturn {
    SQLSetEnvAttrW(environment_handle, attribute, value, _string_length)
}

#[no_mangle]
pub extern "C" fn SQLSetEnvAttrW(
    environment_handle: HEnv,
    attribute: EnvironmentAttribute,
    value: Pointer,
    _string_length: Integer,
) -> SqlReturn {
    let env_handle = MongoHandleRef::from(environment_handle);
    env_handle.clear_diagnostics();
    let env = must_be_valid!(env_handle.as_env());
    match attribute {
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
        EnvironmentAttribute::ConnectionPooling => match FromPrimitive::from_i32(value as i32) {
            Some(ConnectionPooling::Off) => SqlReturn::SUCCESS,
            _ => {
                env_handle.add_diag_info(ODBCError::OptionValueChanged(
                    "SQL_ATTR_CONNECTION_POOLING",
                    "SQL_CP_OFF",
                ));
                SqlReturn::SUCCESS_WITH_INFO
            }
        },
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
    }
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
    hstmt: HStmt,
    attr: StatementAttribute,
    value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    let stmt_handle = MongoHandleRef::from(hstmt);
    stmt_handle.clear_diagnostics();
    let stmt = must_be_valid!(stmt_handle.as_statement());
    match attr {
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
        StatementAttribute::CursorScrollable => match FromPrimitive::from_usize(value as usize) {
            Some(CursorScrollable::NonScrollable) => SqlReturn::SUCCESS,
            _ => {
                stmt_handle
                    .add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_CURSOR_SCROLLABLE"));
                SqlReturn::ERROR
            }
        },
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
                None => stmt_handle.add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_NOSCAN")),
            }
            SqlReturn::SUCCESS
        }
        StatementAttribute::ParamBindOffsetPtr => {
            stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_PARAM_BIND_OFFSET_PTR"));
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
            stmt_handle.add_diag_info(ODBCError::Unimplemented("SQL_ATTR_PARAMS_PROCESSED_PTR"));
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
                stmt_handle.add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_RETRIEVE_DATA"));
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
                stmt_handle.add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_ROW_ARRAY_SIZE"));
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
                stmt_handle.add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_USE_BOOKMARKS"));
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

fn sql_tables(
    mongo_connection: &MongoConnection,
    query_timeout: i32,
    catalog: &str,
    schema: &str,
    table: &str,
    _table_t: &str,
) -> Result<Box<dyn MongoStatement>> {
    match (catalog, schema, table) {
        ("SQL_ALL_CATALOGS", "", "") => Ok(Box::new(MongoDatabases::list_all_catalogs(
            mongo_connection,
            Some(query_timeout),
        ))),
        (_, _, _) => Err(ODBCError::Unimplemented("sql_tables")),
    }
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
    let mongo_handle = MongoHandleRef::from(statement_handle);
    let stmt = must_be_valid!((*mongo_handle).as_statement());
    let catalog = input_wtext_to_string(catalog_name, name_length_1 as usize);
    let schema = input_wtext_to_string(schema_name, name_length_2 as usize);
    let table = input_wtext_to_string(table_name, name_length_3 as usize);
    let table_t = input_wtext_to_string(table_type, name_length_4 as usize);
    let connection = (*(stmt.read().unwrap())).connection;
    let mongo_statement = unsafe {
        sql_tables(
            (*connection)
                .as_connection()
                .unwrap()
                .read()
                .unwrap()
                .mongo_connection
                .as_ref()
                .unwrap(),
            (*(stmt.read().unwrap())).attributes.query_timeout as i32,
            &catalog,
            &schema,
            &table,
            &table_t,
        )
    };
    let mongo_statement = odbc_unwrap!(mongo_statement, mongo_handle);
    stmt.write().unwrap().mongo_statement = Some(mongo_statement);
    SqlReturn::SUCCESS
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
        if output_ptr.is_null() {
            return;
        }
        let sql_state = &format!("{}\0", sql_state);
        let state_u16 = sql_state.encode_utf16().collect::<Vec<u16>>();
        unsafe {
            copy_nonoverlapping(state_u16.as_ptr(), output_ptr, 6);
        }
    }

    /// set_output_string writes [`message`] to the [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the message should be truncated
    /// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    pub fn set_output_string(
        message: &str,
        output_ptr: *mut WChar,
        buffer_len: usize,
        text_length_ptr: *mut SmallInt,
    ) -> SqlReturn {
        unsafe {
            if output_ptr.is_null() {
                if !text_length_ptr.is_null() {
                    *text_length_ptr = 0 as SmallInt;
                } else {
                    // If the output_ptr is NULL, we should still return the length of the message.
                    let message_u16 = message.encode_utf16().collect::<Vec<u16>>();
                    *text_length_ptr = message_u16.len() as SmallInt;
                }
                return SqlReturn::SUCCESS_WITH_INFO;
            }
            // Check if the entire message plus a null terminator can fit in the buffer;
            // we should truncate the message if it's too long.
            let mut message_u16 = message.encode_utf16().collect::<Vec<u16>>();
            let message_len = message_u16.len();
            let num_chars = min(message_len + 1, buffer_len);
            // It is possible that no buffer space has been allocated.
            if num_chars == 0 {
                return SqlReturn::SUCCESS_WITH_INFO;
            }
            message_u16.resize(num_chars - 1, 0);
            message_u16.push('\u{0}' as u16);
            copy_nonoverlapping(message_u16.as_ptr(), output_ptr, num_chars);
            // Store the number of characters in the message string, excluding the
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
        let message = format!("{}", error);
        set_output_string(
            &message,
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

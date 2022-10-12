use crate::util::set_output_fixed_data;
use crate::{
    api::{
        definitions::*,
        errors::{ODBCError, Result},
        functions::util::{
            input_wtext_to_string, set_output_string, set_str_length, unsupported_function,
        },
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
    SqlReturn, StatementAttribute, UInteger, ULen, USmallInt, WChar,
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

macro_rules! must_be_env {
    ($handle:expr) => {{
        let env = (*$handle).as_env();
        must_be_valid!(env)
    }};
}

macro_rules! must_be_conn {
    ($handle:expr) => {{
        let conn = (*$handle).as_connection();
        must_be_valid!(conn)
    }};
}

macro_rules! must_be_stmt {
    ($handle:expr) => {{
        let stmt = (*$handle).as_statement();
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

///
/// [`SQLAllocHandle`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLAllocHandle-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLAllocHandle(
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

///
/// [`SQLBindCol`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLBindCol-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLBindCol(
    _hstmt: HStmt,
    _col_number: USmallInt,
    _target_type: CDataType,
    _target_value: Pointer,
    _buffer_length: Len,
    _length_or_indicatior: *mut Len,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLBindParameter`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLBindParameter-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLBindParameter(
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

///
/// [`SQLBrowseConnect`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLBrowseConnect-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLBrowseConnect(
    connection_handle: HDbc,
    _in_connection_string: *const Char,
    _string_length: SmallInt,
    _out_connection_string: *mut Char,
    _buffer_length: SmallInt,
    _out_buffer_length: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLBrowseConnect")
}

///
/// [`SQLBrowseConnectW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLBrowseConnect-function
///
/// This is the WChar version of the SQLBrowseConnect function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLBrowseConnectW(
    _connection_handle: HDbc,
    _in_connection_string: *const WChar,
    _string_length: SmallInt,
    _out_connection_string: *mut WChar,
    _buffer_length: SmallInt,
    _out_buffer_length: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLBulkOperations`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLBulkOperations-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLBulkOperations(
    statement_handle: HStmt,
    _operation: BulkOperation,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLBulkOperations")
}

///
/// [`SQLCancel`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLCancel-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLCancel(_statement_handle: HStmt) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLCancelHandle`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLCancelHandle-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLCancelHandle(_handle_type: HandleType, _handle: Handle) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLCloseCursor`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLCloseCursor-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLCloseCursor(_statement_handle: HStmt) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLColAttribute`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLColAttribute-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLColAttribute(
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

///
/// [`SQLColAttributeW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLColAttribute-function
///
/// This is the WChar version of the SQLColAttribute function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLColAttributeW(
    statement_handle: HStmt,
    column_number: USmallInt,
    field_identifier: Desc,
    character_attribute_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
    numeric_attribute_ptr: *mut Len,
) -> SqlReturn {
    let string_col_attr = |f: &dyn Fn(&MongoColMetadata) -> &str| {
        let mongo_handle = MongoHandleRef::from(statement_handle);
        let stmt = must_be_valid!((*mongo_handle).as_statement());
        {
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
            if let Ok(col_metadata) = col_metadata {
                return set_output_string(
                    (*f)(col_metadata),
                    character_attribute_ptr as *mut WChar,
                    buffer_length as usize,
                    string_length_ptr,
                );
            }
        }
        // unfortunately, we cannot use odbc_unwrap! on the value because it causes a deadlock.
        mongo_handle.add_diag_info(ODBCError::InvalidDescriptorIndex(column_number));
        SqlReturn::ERROR
    };
    let numeric_col_attr = |f: &dyn Fn(&MongoColMetadata) -> Len| {
        let mongo_handle = MongoHandleRef::from(statement_handle);
        let stmt = must_be_valid!((*mongo_handle).as_statement());
        {
            let stmt_contents = stmt.read().unwrap();
            if stmt_contents.mongo_statement.is_none() {
                *numeric_attribute_ptr = 0 as Len;
                return SqlReturn::SUCCESS;
            }
            let col_metadata = stmt_contents
                .mongo_statement
                .as_ref()
                .unwrap()
                .get_col_metadata(column_number);
            if let Ok(col_metadata) = col_metadata {
                *numeric_attribute_ptr = (*f)(col_metadata);
                return SqlReturn::SUCCESS;
            }
        }
        // unfortunately, we cannot use odbc_unwrap! on the value because it causes a deadlock.
        mongo_handle.add_diag_info(ODBCError::InvalidDescriptorIndex(column_number));
        SqlReturn::ERROR
    };
    match field_identifier {
        Desc::AutoUniqueValue => {
            *numeric_attribute_ptr = SqlBool::False as Len;
            SqlReturn::SUCCESS
        }
        Desc::Unnamed | Desc::Updatable => {
            *numeric_attribute_ptr = 0 as Len;
            SqlReturn::SUCCESS
        }
        Desc::Count => {
            let mongo_handle = MongoHandleRef::from(statement_handle);
            let stmt = must_be_valid!((*mongo_handle).as_statement());
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
            SqlReturn::SUCCESS
        }
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
            let mongo_handle = MongoHandleRef::from(statement_handle);
            let _ = must_be_valid!((*mongo_handle).as_statement());
            mongo_handle
                .add_diag_info(ODBCError::UnsupportedFieldDescriptor(format!("{:?}", desc)));
            SqlReturn::ERROR
        }
    }
}

///
/// [`SQLColumnPrivileges`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLColumnPrivileges-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLColumnPrivileges(
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

///
/// [`SQLColumnPrivilegesW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLColumnPrivileges-function
///
/// This is the WChar version of the SQLColumnPrivileges function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLColumnPrivilegesW(
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

///
/// [`SQLColumns`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLColumns-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLColumns(
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

///
/// [`SQLColumnsW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLColumns-function
///
/// This is the WChar version of the SQLColumns function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLColumnsW(
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

///
/// [`SQLCompleteAsync`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLCompleteAsync-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLCompleteAsync(
    _handle_type: HandleType,
    handle: Handle,
    _async_ret_code_ptr: *mut RetCode,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(handle), "SQLCompleteAsync")
}

///
/// [`SQLConnect`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLConnect-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLConnect(
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

///
/// [`SQLConnectW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLConnect-function
///
/// This is the WChar version of the SQLConnect function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLConnectW(
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

///
/// [`SQLCopyDesc`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLCopyDesc-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLCopyDesc(
    _source_desc_handle: HDesc,
    _target_desc_handle: HDesc,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLDataSources`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDataSources-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLDataSources(
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

///
/// [`SQLDataSourcesW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDataSources-function
///
/// This is the WChar version of the SQLDataSources function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLDataSourcesW(
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

///
/// [`SQLDescribeCol`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDescribeCol-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLDescribeCol(
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

///
/// [`SQLDescribeColW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDescribeCol-function
///
/// This is the WChar version of the SQLDescribeCol function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLDescribeColW(
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

///
/// [`SQLDescribeParam`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDescribeParam-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLDescribeParam(
    statement_handle: HStmt,
    _parameter_number: USmallInt,
    _data_type_ptr: *mut SqlDataType,
    _parameter_size_ptr: *mut ULen,
    _decimal_digits_ptr: *mut SmallInt,
    _nullable_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLDescribeParam")
}

///
/// [`SQLDisconnect`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDisconnect-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLDisconnect(connection_handle: HDbc) -> SqlReturn {
    let conn_handle = MongoHandleRef::from(connection_handle);
    let conn = must_be_valid!((*conn_handle).as_connection());
    // set the mongo_connection to None. This will cause the previous mongo_connection
    // to drop and disconnect.
    conn.write().unwrap().mongo_connection = None;
    SqlReturn::SUCCESS
}

///
/// [`SQLDriverConnect`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDriverConnect-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLDriverConnect(
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

///
/// [`SQLDriverConnectW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDriverConnect-function
///
/// This is the WChar version of the SQLDriverConnect function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLDriverConnectW(
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

///
/// [`SQLDrivers`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDrivers-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLDrivers(
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

///
/// [`SQLDriversW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDrivers-function
///
/// This is the WChar version of the SQLDrivers function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLDriversW(
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

///
/// [`SQLEndTran`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLEndTran-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLEndTran(
    _handle_type: HandleType,
    _handle: Handle,
    _completion_type: CompletionType,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLExecDirect`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLExecDirect-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLExecDirect(
    statement_handle: HStmt,
    _statement_text: *const Char,
    _text_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLExecDirect")
}

///
/// [`SQLExecDirectW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLExecDirect-function
///
/// This is the WChar version of the SQLExecDirect function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLExecDirectW(
    statement_handle: HStmt,
    _statement_text: *const WChar,
    _text_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLExecDirectW")
}

///
/// [`SQLExecute`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLExecute-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLExecute(statement_handle: HStmt) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLExecute")
}

///
/// [`SQLFetch`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLFetch-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLFetch(_statement_handle: HStmt) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLFetchScroll`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLFetchScroll-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLFetchScroll(
    _statement_handle: HStmt,
    _fetch_orientation: FetchOrientation,
    _fetch_offset: Len,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLForeignKeys`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLForeignKeys-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLForeignKeys(
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

///
/// [`SQLForeignKeysW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLForeignKeys-function
///
/// This is the WChar version of the SQLForeignKeys function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLForeignKeysW(
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

///
/// [`SQLFreeHandle`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLFreeHandle-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLFreeHandle(handle_type: HandleType, handle: Handle) -> SqlReturn {
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

///
/// [`SQLFreeStmt`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLFreeStmt-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLFreeStmt(_statement_handle: HStmt, _option: SmallInt) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLGetConnectAttr`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetConnectAttr-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetConnectAttr(
    connection_handle: HDbc,
    _attribute: ConnectionAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLGetConnectAttr")
}

///
/// [`SQLGetConnectAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetConnectAttr-function
///
/// This is the WChar version of the SQLGetConnectAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetConnectAttrW(
    connection_handle: HDbc,
    attribute: ConnectionAttribute,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    let conn_handle = MongoHandleRef::from(connection_handle);
    let conn = must_be_valid!((*conn_handle).as_connection());
    let attributes = &conn.read().unwrap().attributes;

    match attribute {
        ConnectionAttribute::CurrentCatalog => {
            let current_catalog = attributes.current_catalog.map(|s| s.as_str());
            match current_catalog {
                None => SqlReturn::NO_DATA,
                Some(cc) => set_output_string(
                    cc,
                    value_ptr as *mut WChar,
                    buffer_length as usize,
                    string_length_ptr as *mut SmallInt,
                ),
            }
        }
        ConnectionAttribute::LoginTimeout => {
            let login_timeout = attributes.login_timeout.unwrap_or(0);
            set_output_fixed_data(&login_timeout, value_ptr, string_length_ptr as *mut Len)
        }
        ConnectionAttribute::ConnectionTimeout => {
            let connection_timeout = attributes.connection_timeout.unwrap_or(0);
            set_output_fixed_data(
                &connection_timeout,
                value_ptr,
                string_length_ptr as *mut Len,
            )
        }
        _ => {
            conn_handle.add_diag_info(ODBCError::InvalidAttrIdentifier(attribute));
            SqlReturn::ERROR
        }
    }
}

///
/// [`SQLGetCursorName`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetCursorName-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetCursorName(
    statement_handle: HStmt,
    _cursor_name: *mut Char,
    _buffer_length: SmallInt,
    _name_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLGetCursorName")
}

///
/// [`SQLGetCursorNameW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetCursorName-function
///
/// This is the WChar version of the SQLGetCursorName function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetCursorNameW(
    _statement_handle: HStmt,
    _cursor_name: *mut WChar,
    _buffer_length: SmallInt,
    _name_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLGetData`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetData-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetData(
    _statement_handle: HStmt,
    _col_or_param_num: USmallInt,
    _target_type: CDataType,
    _target_value_ptr: Pointer,
    _buffer_length: Len,
    _str_len_or_ind_ptr: *mut Len,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLGetDescField`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDescField-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetDescField(
    _descriptor_handle: HDesc,
    _record_number: SmallInt,
    _field_identifier: SmallInt,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLGetDescFieldW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDescField-function
///
/// This is the WChar version of the SQLGetDescField function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetDescFieldW(
    _descriptor_handle: HDesc,
    _record_number: SmallInt,
    _field_identifier: SmallInt,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLGetDescRec`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDescRec-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetDescRec(
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

///
/// [`SQLGetDescRecW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDescRec-function
///
/// This is the WChar version of the SQLGetDescRec function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetDescRecW(
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

///
/// [`SQLGetDiagField`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDiagField-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetDiagField(
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

///
/// [`SQLGetDiagFieldW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDiagField-function
///
/// This is the WChar version of the SQLGetDiagField function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetDiagFieldW(
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

///
/// [`SQLGetDiagRec`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDiagRec-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetDiagRec(
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

///
/// [`SQLGetDiagRecW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDiagRec-function
///
/// This is the WChar version of the SQLGetDiagRec function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetDiagRecW(
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
            let env = must_be_env!(mongo_handle);
            get_error(&(*env).read().unwrap().errors)
        }
        HandleType::Dbc => {
            let dbc = must_be_conn!(mongo_handle);
            get_error(&(*dbc).read().unwrap().errors)
        }
        HandleType::Stmt => {
            let stmt = must_be_stmt!(mongo_handle);
            get_error(&(*stmt).read().unwrap().errors)
        }
        HandleType::Desc => unimplemented!(),
    }
}

///
/// [`SQLGetEnvAttr`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetEnvAttr-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetEnvAttr(
    environment_handle: HEnv,
    _attribute: EnvironmentAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length: *mut Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(environment_handle), "SQLGetEnvAttr")
}

///
/// [`SQLGetEnvAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetEnvAttr-function
///
/// This is the WChar version of the SQLGetEnvAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetEnvAttrW(
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
            EnvironmentAttribute::OdbcVersion => {
                *(value_ptr as *mut OdbcVersion) = env_contents.attributes.odbc_ver;
            }
            EnvironmentAttribute::OutputNts => {
                *(value_ptr as *mut SqlBool) = env_contents.attributes.output_nts;
            }
            EnvironmentAttribute::ConnectionPooling => {
                *(value_ptr as *mut ConnectionPooling) = env_contents.attributes.connection_pooling;
            }
            EnvironmentAttribute::CpMatch => {
                *(value_ptr as *mut CpMatch) = env_contents.attributes.cp_match;
            }
        }
    }
    SqlReturn::SUCCESS
}

///
/// [`SQLGetInfo`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetInfo-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetInfo(
    connection_handle: HDbc,
    _info_type: InfoType,
    _info_value_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLGetInfo")
}

///
/// [`SQLGetInfoW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetInfo-function
///
/// This is the WChar version of the SQLGetInfo function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetInfoW(
    connection_handle: HDbc,
    _info_type: InfoType,
    _info_value_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLGetInfoW")
}

///
/// [`SQLGetStmtAttr`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetStmtAttr-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetStmtAttr(
    handle: HStmt,
    _attribute: StatementAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(handle), "SQLGetStmtAttr")
}

///
/// [`SQLGetStmtAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetStmtAttr-function
///
/// This is the WChar version of the SQLGetStmtAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetStmtAttrW(
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
        StatementAttribute::AppRowDesc => {
            *(value_ptr as *mut Pointer) = stmt_contents.attributes.app_row_desc;
            set_str_length(string_length_ptr, size_of::<Pointer>() as Integer);
        }
        StatementAttribute::AppParamDesc => {
            *(value_ptr as *mut Pointer) = stmt_contents.attributes.app_param_desc;
            set_str_length(string_length_ptr, size_of::<Pointer>() as Integer);
        }
        StatementAttribute::ImpRowDesc => {
            *(value_ptr as *mut Pointer) = stmt_contents.attributes.imp_row_desc;
            set_str_length(string_length_ptr, size_of::<Pointer>() as Integer);
        }
        StatementAttribute::ImpParamDesc => {
            *(value_ptr as *mut Pointer) = stmt_contents.attributes.imp_param_desc;
            set_str_length(string_length_ptr, size_of::<Pointer>() as Integer);
        }
        StatementAttribute::FetchBookmarkPtr => {
            *(value_ptr as *mut _) = stmt_contents.attributes.fetch_bookmark_ptr;
            set_str_length(string_length_ptr, size_of::<*mut Len>() as Integer);
        }
        StatementAttribute::CursorScrollable => {
            *(value_ptr as *mut CursorScrollable) = stmt_contents.attributes.cursor_scrollable;
        }
        StatementAttribute::CursorSensitivity => {
            *(value_ptr as *mut CursorSensitivity) = stmt_contents.attributes.cursor_sensitivity;
        }
        StatementAttribute::AsyncEnable => {
            *(value_ptr as *mut AsyncEnable) = stmt_contents.attributes.async_enable;
        }
        StatementAttribute::Concurrency => {
            *(value_ptr as *mut Concurrency) = stmt_contents.attributes.concurrency;
        }
        StatementAttribute::CursorType => {
            *(value_ptr as *mut CursorType) = stmt_contents.attributes.cursor_type;
        }
        StatementAttribute::EnableAutoIpd => {
            *(value_ptr as *mut SqlBool) = stmt_contents.attributes.enable_auto_ipd;
        }
        StatementAttribute::KeysetSize => {
            *(value_ptr as *mut ULen) = 0;
        }
        StatementAttribute::MaxLength => {
            *(value_ptr as *mut ULen) = stmt_contents.attributes.max_length;
        }
        StatementAttribute::MaxRows => {
            *(value_ptr as *mut ULen) = stmt_contents.attributes.max_rows;
        }
        StatementAttribute::NoScan => {
            *(value_ptr as *mut NoScan) = stmt_contents.attributes.no_scan;
        }
        StatementAttribute::ParamBindOffsetPtr => {
            *(value_ptr as *mut _) = stmt_contents.attributes.param_bind_offset_ptr;
            set_str_length(string_length_ptr, size_of::<*mut ULen>() as Integer)
        }
        StatementAttribute::ParamBindType => {
            *(value_ptr as *mut ULen) = stmt_contents.attributes.param_bind_type;
        }
        StatementAttribute::ParamOpterationPtr => {
            *(value_ptr as *mut _) = stmt_contents.attributes.param_operation_ptr;
            set_str_length(string_length_ptr, size_of::<*mut USmallInt>() as Integer)
        }
        StatementAttribute::ParamStatusPtr => {
            *(value_ptr as *mut _) = stmt_contents.attributes.param_status_ptr;
            set_str_length(string_length_ptr, size_of::<*mut USmallInt>() as Integer)
        }
        StatementAttribute::ParamsProcessedPtr => {
            *(value_ptr as *mut _) = stmt_contents.attributes.param_processed_ptr;
            set_str_length(string_length_ptr, size_of::<*mut ULen>() as Integer)
        }
        StatementAttribute::ParamsetSize => {
            *(value_ptr as *mut ULen) = stmt_contents.attributes.paramset_size;
        }
        StatementAttribute::QueryTimeout => {
            *(value_ptr as *mut ULen) = stmt_contents.attributes.query_timeout;
        }
        StatementAttribute::RetrieveData => {
            *(value_ptr as *mut RetrieveData) = stmt_contents.attributes.retrieve_data;
        }
        StatementAttribute::RowBindOffsetPtr => {
            *(value_ptr as *mut _) = stmt_contents.attributes.row_bind_offset_ptr;
            set_str_length(string_length_ptr, size_of::<*mut ULen>() as Integer)
        }
        StatementAttribute::RowBindType => {
            *(value_ptr as *mut ULen) = stmt_contents.attributes.row_bind_type;
        }
        StatementAttribute::RowNumber => {
            *(value_ptr as *mut ULen) = stmt_contents.attributes.row_number;
        }
        StatementAttribute::RowOperationPtr => {
            *(value_ptr as *mut _) = stmt_contents.attributes.row_operation_ptr;
            set_str_length(string_length_ptr, size_of::<*mut USmallInt>() as Integer)
        }
        StatementAttribute::RowStatusPtr => {
            *(value_ptr as *mut _) = stmt_contents.attributes.row_status_ptr;
            set_str_length(string_length_ptr, size_of::<*mut USmallInt>() as Integer)
        }
        StatementAttribute::RowsFetchedPtr => {
            *(value_ptr as *mut _) = stmt_contents.attributes.rows_fetched_ptr;
            set_str_length(string_length_ptr, size_of::<*mut ULen>() as Integer)
        }
        StatementAttribute::RowArraySize => {
            *(value_ptr as *mut ULen) = stmt_contents.attributes.row_array_size;
        }
        StatementAttribute::SimulateCursor => {
            *(value_ptr as *mut ULen) = stmt_contents.attributes.simulate_cursor;
        }
        StatementAttribute::UseBookmarks => {
            *(value_ptr as *mut UseBookmarks) = stmt_contents.attributes.use_bookmarks;
        }
        StatementAttribute::AsyncStmtEvent => {
            *(value_ptr as *mut _) = stmt_contents.attributes.async_stmt_event;
        }
        StatementAttribute::MetadataId => {
            todo!();
        }
    }
    SqlReturn::SUCCESS
}

///
/// [`SQLGetTypeInfo`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetTypeInfo-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLGetTypeInfo(_handle: HStmt, _data_type: SqlDataType) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLMoreResults`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLMoreResults-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLMoreResults(_handle: HStmt) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLNativeSql`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLNativeSql-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLNativeSql(
    connection_handle: HDbc,
    _in_statement_text: *const Char,
    _in_statement_len: Integer,
    _out_statement_text: *mut Char,
    _buffer_len: Integer,
    _out_statement_len: *mut Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(connection_handle), "SQLNativeSql")
}

///
/// [`SQLNativeSqlW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLNativeSql-function
///
/// This is the WChar version of the SQLNativeSql function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLNativeSqlW(
    _connection_handle: HDbc,
    _in_statement_text: *const WChar,
    _in_statement_len: Integer,
    _out_statement_text: *mut WChar,
    _buffer_len: Integer,
    _out_statement_len: *mut Integer,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLNumParams`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLNumParams-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLNumParams(
    statement_handle: HStmt,
    _param_count_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLNumParams")
}

///
/// [`SQLNumResultCols`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLNumResultCols-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLNumResultCols(
    statement_handle: HStmt,
    column_count_ptr: *mut SmallInt,
) -> SqlReturn {
    let mongo_handle = MongoHandleRef::from(statement_handle);
    let stmt = must_be_valid!((*mongo_handle).as_statement());
    let stmt_contents = stmt.read().unwrap();
    let mongo_statement = stmt_contents.mongo_statement.as_ref();
    if mongo_statement.is_none() {
        *column_count_ptr = 0;
        return SqlReturn::SUCCESS;
    }
    *column_count_ptr = mongo_statement.unwrap().get_resultset_metadata().len() as SmallInt;
    SqlReturn::SUCCESS
}

///
/// [`SQLParamData`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLParamData-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLParamData(hstmt: HStmt, _value_ptr_ptr: *mut Pointer) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hstmt), "SQLParamData")
}

///
/// [`SQLPrepare`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLPrepare-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLPrepare(
    hstmt: HStmt,
    _statement_text: *const Char,
    _text_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hstmt), "SQLPrepare")
}

///
/// [`SQLPrepareW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLPrepare-function
///
/// This is the WChar version of the SQLPrepare function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLPrepareW(
    hstmt: HStmt,
    _statement_text: *const WChar,
    _text_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hstmt), "SQLPrepareW")
}

///
/// [`SQLPrimaryKeys`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLPrimaryKeys-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLPrimaryKeys(
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

///
/// [`SQLPrimaryKeysW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLPrimaryKeys-function
///
/// This is the WChar version of the SQLPrimaryKeys function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLPrimaryKeysW(
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

///
/// [`SQLProcedureColumns`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLProcedureColumns-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLProcedureColumns(
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

///
/// [`SQLProcedureColumnsW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLProcedureColumns-function
///
/// This is the WChar version of the SQLProcedureColumns function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLProcedureColumnsW(
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

///
/// [`SQLProcedures`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLProcedures-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLProcedures(
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

///
/// [`SQLProceduresW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLProcedures-function
///
/// This is the WChar version of the SQLProcedures function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLProceduresW(
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

///
/// [`SQLPutData`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLPutData-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLPutData(
    statement_handle: HStmt,
    _data_ptr: Pointer,
    _str_len_or_ind_ptr: Len,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLPutData")
}

///
/// [`SQLRowCount`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLRowCount-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLRowCount(
    statement_handle: HStmt,
    row_count_ptr: *mut Len,
) -> SqlReturn {
    let mongo_handle = MongoHandleRef::from(statement_handle);
    // even though we always return 0, we must still assert that the proper handle
    // type is sent by the client.
    let _ = must_be_valid!((*mongo_handle).as_statement());
    *row_count_ptr = 0 as Len;
    SqlReturn::SUCCESS
}

///
/// [`SQLSetConnectAttr`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetConnectAttr-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetConnectAttr(
    hdbc: HDbc,
    attr: ConnectionAttribute,
    value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    let conn_handle = MongoHandleRef::from(hdbc);
    let conn = must_be_valid!((*conn_handle).as_connection());

    match attr {
        ConnectionAttribute::LoginTimeout => match FromPrimitive::from_u32(value as u32) {
            Some(login_timeout) => {
                let mut c = conn.write().unwrap();
                c.attributes.login_timeout = Some(login_timeout);
                SqlReturn::SUCCESS
            }
            None => {
                env_handle.add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_LOGIN_TIMEOUT"));
                SqlReturn::ERROR
            }
        },
        // For now, since PowerBI does not use these we omit setting them
        ConnectionAttribute::ConnectionTimeout | ConnectionAttribute::CurrentCatalog => {
            SqlReturn::SUCCESS
        }
        _ => {
            conn_handle.add_diag_info(ODBCError::InvalidAttrIdentifier(attribute));
            SqlReturn::ERROR
        }
    }
}

///
/// [`SQLSetConnectAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetConnectAttr-function
///
/// This is the WChar version of the SQLSetConnectAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetConnectAttrW(
    _hdbc: HDbc,
    _attr: ConnectionAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLSetCursorName`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetCursorName-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetCursorName(
    statement_handle: HStmt,
    _cursor_name: *const Char,
    _name_length: SmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLSetCursorName")
}

///
/// [`SQLSetCursorNameW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetCursorName-function
///
/// This is the WChar version of the SQLSetCursorName function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetCursorNameW(
    _statement_handle: HStmt,
    _cursor_name: *const WChar,
    _name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLSetDescField`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetDescField-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetDescField(
    _desc_handle: HDesc,
    _rec_number: SmallInt,
    _field_identifier: SmallInt,
    _value_ptr: Pointer,
    _buffer_length: Integer,
) -> SqlReturn {
    unimplemented!()
}

///
/// [`SQLSetDescRec`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetDescRec-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetDescRec(
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

///
/// [`SQLSetPos`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetPos-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetPos(
    statement_handle: HStmt,
    _row_number: ULen,
    _operation: USmallInt,
    _lock_type: USmallInt,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(statement_handle), "SQLSetPos")
}

///
/// [`SQLSetEnvAttr`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetEnvAttr-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetEnvAttr(
    environment_handle: HEnv,
    attribute: EnvironmentAttribute,
    value: Pointer,
    _string_length: Integer,
) -> SqlReturn {
    SQLSetEnvAttrW(environment_handle, attribute, value, _string_length)
}

///
/// [`SQLSetEnvAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetEnvAttr-function
///
/// This is the WChar version of the SQLSetEnvAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetEnvAttrW(
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

///
/// [`SQLSetStmtAttr`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetStmtAttr-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetStmtAttr(
    hstmt: HStmt,
    _attr: StatementAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    unsupported_function(MongoHandleRef::from(hstmt), "SQLSetStmtAttr")
}

///
/// [`SQLSetStmtAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetStmtAttr-function
///
/// This is the WChar version of the SQLSetStmtAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSetStmtAttrW(
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

///
/// [`SQLSpecialColumns`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSpecialColumns-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSpecialColumns(
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

///
/// [`SQLSpecialColumnsW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSpecialColumns-function
///
/// This is the WChar version of the SQLSpecialColumns function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLSpecialColumnsW(
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

///
/// [`SQLStatistics`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLStatistics-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLStatistics(
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

///
/// [`SQLTablePrivileges`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLTablePrivileges-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLTablePrivileges(
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

///
/// [`SQLTablesPrivilegesW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLTablesPrivileges-function
///
/// This is the WChar version of the SQLTablesPrivileges function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLTablesPrivilegesW(
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

///
/// [`SQLTables`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLTables-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLTables(
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

///
/// [`SQLTablesW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLTables-function
///
/// This is the WChar version of the SQLTables function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLTablesW(
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
    let mongo_statement = sql_tables(
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
    );
    let mongo_statement = odbc_unwrap!(mongo_statement, mongo_handle);
    stmt.write().unwrap().mongo_statement = Some(mongo_statement);
    SqlReturn::SUCCESS
}

pub mod util {
    use crate::{api::errors::ODBCError, handles::definitions::MongoHandle};
    use odbc_sys::{Integer, Len, Pointer, SmallInt, SqlReturn, WChar};
    use std::{cmp::min, mem::size_of, ptr::copy_nonoverlapping};

    ///
    /// input_wtext_to_string converts an input cstring to a rust String.
    /// It assumes nul termination if the supplied length is negative.
    ///
    /// # Safety
    /// This converts raw C-pointers to rust Strings, which requires unsafe operations
    ///
    #[allow(clippy::uninit_vec)]
    pub unsafe fn input_wtext_to_string(text: *const WChar, len: usize) -> String {
        if (len as isize) < 0 {
            let mut dst = Vec::new();
            let mut itr = text;
            {
                while *itr != 0 {
                    dst.push(*itr);
                    itr = itr.offset(1);
                }
            }
            return String::from_utf16_lossy(&dst);
        }

        let mut dst = Vec::with_capacity(len);
        dst.set_len(len);
        copy_nonoverlapping(text, dst.as_mut_ptr(), len);
        String::from_utf16_lossy(&dst)
    }

    ///
    /// set_sql_state writes the given sql state to the [`output_ptr`].
    ///
    /// # Safety
    /// This writes to a raw C-pointer
    ///
    pub unsafe fn set_sql_state(sql_state: &str, output_ptr: *mut WChar) {
        if output_ptr.is_null() {
            return;
        }
        let sql_state = &format!("{}\0", sql_state);
        let state_u16 = sql_state.encode_utf16().collect::<Vec<u16>>();
        copy_nonoverlapping(state_u16.as_ptr(), output_ptr, 6);
    }

    ///
    /// set_output_string writes [`message`] to the [`output_ptr`]. [`buffer_len`] is the
    /// length of the [`output_ptr`] buffer in characters; the message should be truncated
    /// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
    /// should be stored in [`text_length_ptr`].
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_string(
        message: &str,
        output_ptr: *mut WChar,
        buffer_len: usize,
        text_length_ptr: *mut SmallInt,
    ) -> SqlReturn {
        if output_ptr.is_null() {
            if !text_length_ptr.is_null() {
                *text_length_ptr = 0 as SmallInt;
            } else {
                // If the output_ptr is NULL, we should still return the length of the message.
                *text_length_ptr = message.encode_utf16().count() as i16;
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

    ///
    /// set_output_fixed_data writes [`data`], which must be a fixed sized type, to the Pointer [`output_ptr`].
    /// ODBC drivers assume the output buffer is large enough for fixed types, and are allowed to
    /// overwrite the buffer if too small a buffer is passed.
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn set_output_fixed_data<T: core::fmt::Debug>(
        data: &T,
        output_ptr: Pointer,
        data_len_ptr: *mut Len,
    ) -> SqlReturn {
        if !data_len_ptr.is_null() {
            // If the output_ptr is NULL, we should still return the length of the message.
            *data_len_ptr = size_of::<T>() as isize;
        }
        if output_ptr.is_null() {
            return SqlReturn::SUCCESS_WITH_INFO;
        }
        copy_nonoverlapping(data as *const _, output_ptr as *mut _, 1);
        SqlReturn::SUCCESS
    }

    ///
    /// get_diag_rec copies the given ODBC error's diagnostic information
    /// into the provided pointers.
    ///
    /// # Safety
    /// This writes to multiple raw C-pointers
    ///
    pub unsafe fn get_diag_rec(
        error: &ODBCError,
        state: *mut WChar,
        message_text: *mut WChar,
        buffer_length: SmallInt,
        text_length_ptr: *mut SmallInt,
        native_error_ptr: *mut Integer,
    ) -> SqlReturn {
        if !native_error_ptr.is_null() {
            *native_error_ptr = error.get_native_err_code();
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

    ///
    /// unsupported_function is a helper function for correctly setting the state for
    /// unsupported functions.
    ///
    pub fn unsupported_function(handle: &mut MongoHandle, name: &'static str) -> SqlReturn {
        handle.clear_diagnostics();
        handle.add_diag_info(ODBCError::Unimplemented(name));
        SqlReturn::ERROR
    }

    ///
    /// set_str_length writes the given length to [`string_length_ptr`].
    ///
    /// # Safety
    /// This writes to a raw C-pointers
    ///
    pub unsafe fn set_str_length(string_length_ptr: *mut Integer, length: Integer) {
        if !string_length_ptr.is_null() {
            *string_length_ptr = length
        }
    }
}

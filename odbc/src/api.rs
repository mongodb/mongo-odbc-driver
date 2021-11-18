use odbc_sys::{
    BulkOperation, CDataType, Char, CompletionType, ConnectionAttribute, Desc, DriverConnectOption,
    EnvironmentAttribute, FetchOrientation, HDbc, HDesc, HEnv, HStmt, HWnd, Handle, HandleType,
    InfoType, Integer, Len, Nullability, ParamType, Pointer, RetCode, SmallInt, SqlDataType,
    SqlReturn, StatementAttribute, ULen, USmallInt, WChar,
};
use std::{
    collections::HashSet,
    sync::{Mutex, RwLock},
};

#[derive(Debug)]
struct EnvHandle {
    env: RwLock<Env>,
}

impl EnvHandle {
    fn new() -> Self {
        Self {
            env: RwLock::new(Env {
                attributes: EnvAttributes::default(),
                state: EnvState::Unallocated,
                connections: HashSet::new(),
            }),
        }
    }
}

#[derive(Debug)]
struct Env {
    // attributes for this Env
    pub attributes: EnvAttributes,
    // state of this Env
    pub state: EnvState,
    pub connections: HashSet<*mut ConnectionHandle>,
}

#[derive(Debug)]
pub struct EnvAttributes {
    pub odbc_ver: Integer,
    // TODO: incomplete
}

impl Default for EnvAttributes {
    fn default() -> Self {
        Self { odbc_ver: 3 }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum EnvState {
    Unallocated,
    Allocated,
}

#[derive(Debug)]
struct ConnectionHandle {
    connection: RwLock<Connection>,
}

#[derive(Debug)]
struct Connection {
    // Pointer to the Env from which
    // this Connection was allocated
    pub env: *mut EnvHandle,
    // all the possible Connection settings
    pub attributes: ConnectionAttributes,
    // state of this connection
    pub state: ConnectionState,
    // MongoDB Client for issuing commands
    // pub client: Option<MongoClient>,
    // all Descriptors attached to this Connection
    pub descriptors: HashSet<*mut DescriptorHandle>,
    // all Statements allocated from this Connection
    pub statements: HashSet<*mut StatementHandle>,
}

#[derive(Debug)]
struct ConnectionAttributes {}

impl Default for ConnectionAttributes {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ConnectionState {
    UnallocatedEnvUnallocatedConnection,
    AllocatedEnvUnallocatedConnection,
    AllocatedEnvAllocatedConnection,
    ConnectionFunctionNeedsDataEnv,
    Connected,
    StatementAllocated,
    TransactionInProgress,
}

impl ConnectionHandle {
    fn new(env: *mut EnvHandle) -> Self {
        Self {
            connection: RwLock::new(Connection::new(env)),
        }
    }
}

impl Connection {
    fn new(env: *mut EnvHandle) -> Self {
        Self {
            env,
            attributes: ConnectionAttributes::default(),
            state: ConnectionState::AllocatedEnvUnallocatedConnection,
            descriptors: HashSet::new(),
            statements: HashSet::new(),
        }
    }
}

#[derive(Debug)]
struct StatementHandle {
    stmt: Mutex<Statement>,
}

#[derive(Debug)]
struct Statement {
    pub connection: *mut ConnectionHandle,
    pub attributes: StatementAttributes,
    pub state: StatementState,
    //pub cursor: Option<Box<Peekable<Cursor>>>,
}

#[derive(Debug)]
struct StatementAttributes {}

impl Default for StatementAttributes {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StatementState {
    Unallocated,
    Allocated,
    Prepared,
    PreparedHasResultSet,
    ExecutedNoResultSet,
    ExecutedHasResultSet,
    CursorFetchSet,
    CursorExtendedFetchSet,
    FunctionNeedsDataNoParam,
    FunctionNeedsDataNoPut,
    FunctionNeedsDataPutCalled,
    Executing,
    AsyncCancelled,
}

impl StatementHandle {
    fn new(connection: *mut ConnectionHandle) -> Self {
        Self {
            stmt: Mutex::new(Statement::new(connection)),
        }
    }
}

impl Statement {
    fn new(connection: *mut ConnectionHandle) -> Self {
        Self {
            connection,
            attributes: StatementAttributes::default(),
            state: StatementState::Unallocated,
        }
    }
}

#[derive(Debug)]
struct DescriptorHandle {
    descriptor: RwLock<Descriptor>,
}

impl DescriptorHandle {
    fn new() -> Self {
        Self {
            descriptor: RwLock::new(Descriptor::new()),
        }
    }
}

#[derive(Debug)]
struct Descriptor {
    state: DescriptorState,
}

#[derive(Debug, PartialEq, Eq)]
enum DescriptorState {
    Unallocated,
    ImplicitlyAllocated,
    ExplicitlyAllocated,
}

impl Descriptor {
    fn new() -> Self {
        Self {
            state: DescriptorState::Unallocated,
        }
    }
}

#[no_mangle]
pub extern "C" fn SQLAllocHandle(
    handle_type: HandleType,
    input_handle: Handle,
    output_handle: *mut Handle,
) -> SqlReturn {
    match handle_type {
        HandleType::Env => {
            let env = Box::new(EnvHandle::new());
            env.env.write().unwrap().state = EnvState::Allocated;
            unsafe { *output_handle = Box::into_raw(env) as *mut _ }
        }
        HandleType::Dbc => {
            let env = input_handle as *mut _;
            let conn = Box::new(ConnectionHandle::new(env));
            conn.connection.write().unwrap().state =
                ConnectionState::AllocatedEnvAllocatedConnection;
            unsafe {
                let conn_ptr = Box::into_raw(conn) as *mut _;
                (*env).env.write().unwrap().connections.insert(conn_ptr);
                *output_handle = conn_ptr as *mut _
            }
        }
        HandleType::Stmt => {
            let conn = input_handle as *mut _;
            let stmt = Box::new(StatementHandle::new(conn));
            stmt.stmt.lock().unwrap().state = StatementState::Allocated;
            unsafe {
                let stmt_ptr = Box::into_raw(stmt) as *mut _;
                (*conn)
                    .connection
                    .write()
                    .unwrap()
                    .statements
                    .insert(stmt_ptr);
                *output_handle = stmt_ptr as *mut _
            }
        }
        HandleType::Desc => {
            let desc = Box::new(DescriptorHandle::new());
            desc.descriptor.write().unwrap().state = DescriptorState::ExplicitlyAllocated;
            unsafe { *output_handle = Box::into_raw(desc) as *mut _ }
        }
    }
    SqlReturn::SUCCESS
}

#[test]
fn env_alloc_free() {
    unsafe {
        let mut handle: *mut _ = &mut EnvHandle::new();
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(EnvState::Unallocated, (*handle).env.read().unwrap().state);
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Env,
                std::ptr::null_mut(),
                std::mem::transmute::<*mut *mut EnvHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(EnvState::Allocated, (*handle).env.read().unwrap().state);
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Env,
                std::mem::transmute::<*mut EnvHandle, Handle>(handle),
            )
        );
    }
}

#[test]
fn connection_alloc_free() {
    unsafe {
        let mut env_handle: *mut _ = &mut EnvHandle::new();
        (*env_handle).env.write().unwrap().state = EnvState::Allocated;

        let mut handle: *mut _ = &mut ConnectionHandle::new(std::ptr::null_mut());
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            ConnectionState::AllocatedEnvUnallocatedConnection,
            (*handle).connection.read().unwrap().state
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Dbc,
                env_handle as *mut _,
                std::mem::transmute::<*mut *mut ConnectionHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            ConnectionState::AllocatedEnvAllocatedConnection,
            (*handle).connection.read().unwrap().state
        );
        assert_eq!(1, (*env_handle).env.read().unwrap().connections.len());
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Dbc,
                std::mem::transmute::<*mut ConnectionHandle, Handle>(handle),
            )
        );
        assert_eq!(0, (*env_handle).env.read().unwrap().connections.len());
    }
}

#[test]
fn statement_alloc_free() {
    unsafe {
        let mut env_handle: *mut _ = &mut EnvHandle::new();
        (*env_handle).env.write().unwrap().state = EnvState::Allocated;

        let mut conn_handle: *mut _ = &mut ConnectionHandle::new(env_handle);
        (*conn_handle).connection.write().unwrap().state =
            ConnectionState::AllocatedEnvAllocatedConnection;

        let mut handle: *mut _ = &mut StatementHandle::new(std::ptr::null_mut());
        let handle_ptr: *mut _ = &mut handle;
        assert_eq!(
            StatementState::Unallocated,
            (*handle).stmt.lock().unwrap().state
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLAllocHandle(
                HandleType::Stmt,
                conn_handle as *mut _,
                std::mem::transmute::<*mut *mut StatementHandle, *mut Handle>(handle_ptr),
            )
        );
        assert_eq!(
            StatementState::Allocated,
            (*handle).stmt.lock().unwrap().state
        );
        assert_eq!(
            1,
            (*conn_handle).connection.read().unwrap().statements.len()
        );
        assert_eq!(
            SqlReturn::SUCCESS,
            SQLFreeHandle(
                HandleType::Stmt,
                std::mem::transmute::<*mut StatementHandle, Handle>(handle),
            )
        );
        assert_eq!(
            0,
            (*conn_handle).connection.read().unwrap().statements.len()
        );
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
    _hstmt: HStmt,
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
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLBrowseConnect(
    _connection_handle: HDbc,
    _in_connection_string: *const Char,
    _string_length: SmallInt,
    _out_connection_string: *mut Char,
    _buffer_length: SmallInt,
    _out_buffer_length: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
    _operation: BulkOperation,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _table_name: *const Char,
    _table_name_length: SmallInt,
    _column_name: *const Char,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _table_name: *const Char,
    _table_name_length: SmallInt,
    _column_name: *const Char,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
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
    _handle: Handle,
    _async_ret_code_ptr: *mut RetCode,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLConnect(
    _connection_handle: HDbc,
    _server_name: *const Char,
    _name_length_1: SmallInt,
    _user_name: *const Char,
    _name_length_2: SmallInt,
    _authentication: *const Char,
    _name_length_3: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLConnectW(
    _connection_handle: HDbc,
    _server_name: *const WChar,
    _name_length_1: SmallInt,
    _user_name: *const WChar,
    _name_length_2: SmallInt,
    _authentication: *const WChar,
    _name_length_3: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLCopyDesc(_source_desc_handle: HDesc, _target_desc_handle: HDesc) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLDataSources(
    _environment_handle: HEnv,
    _direction: FetchOrientation,
    _server_name: *mut Char,
    _buffer_length_1: SmallInt,
    _name_length_1: *mut SmallInt,
    _description: *mut Char,
    _buffer_length_2: SmallInt,
    _name_length_2: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLDataSourcesW(
    _environment_handle: HEnv,
    _direction: FetchOrientation,
    _server_name: *mut WChar,
    _buffer_length_1: SmallInt,
    _name_length_1: *mut SmallInt,
    _description: *mut WChar,
    _buffer_length_2: SmallInt,
    _name_length_2: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLDescribeCol(
    _hstmt: HStmt,
    _col_number: USmallInt,
    _col_name: *mut Char,
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
    _statement_handle: HStmt,
    _parameter_number: USmallInt,
    _data_type_ptr: *mut SqlDataType,
    _parameter_size_ptr: *mut ULen,
    _decimal_digits_ptr: *mut SmallInt,
    _nullable_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLDisconnect(_connection_handle: HDbc) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLDriverConnect(
    _connection_handle: HDbc,
    _window_handle: HWnd,
    _in_connection_string: *const Char,
    _string_length_1: SmallInt,
    _out_connection_string: *mut Char,
    _buffer_length: SmallInt,
    _string_length_2: *mut SmallInt,
    _drive_completion: DriverConnectOption,
) -> SqlReturn {
    unimplemented!()
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
    _henv: HEnv,
    _direction: FetchOrientation,
    _driver_desc: *mut Char,
    _driver_desc_max: SmallInt,
    _out_driver_desc: *mut SmallInt,
    _driver_attributes: *mut Char,
    _drvr_attr_max: SmallInt,
    _out_drvr_attr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLDriversW(
    _henv: HEnv,
    _direction: FetchOrientation,
    _driver_desc: *mut WChar,
    _driver_desc_max: SmallInt,
    _out_driver_desc: *mut SmallInt,
    _driver_attributes: *mut WChar,
    _drvr_attr_max: SmallInt,
    _out_drvr_attr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
    _statement_text: *const Char,
    _text_length: Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLExecDirectW(
    _statement_handle: HStmt,
    _statement_text: *const WChar,
    _text_length: Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLExecute(_statement_handle: HStmt) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
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
    unimplemented!()
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
    unsafe {
        match handle_type {
            // By making Boxes to the types and letting them go out of
            // scope, they will be dropped.
            HandleType::Env => {
                let env = Box::from_raw(handle as *mut EnvHandle);
                // Actually reading this value would make ASAN fail, but this
                // is what the ODBC standard expects.
                env.env.write().unwrap().state = EnvState::Unallocated;
            }
            HandleType::Dbc => {
                let handle = handle as *mut ConnectionHandle;
                let conn = Box::from_raw(handle);
                let mut contents = conn.connection.write().unwrap();
                // Actually reading this value would make ASAN fail, but this
                // is what the ODBC standard expects.
                contents.state = ConnectionState::AllocatedEnvUnallocatedConnection;
                (*contents.env)
                    .env
                    .write()
                    .unwrap()
                    .connections
                    .remove(&handle);
            }
            HandleType::Stmt => {
                let handle = handle as *mut StatementHandle;
                let stmt = Box::from_raw(handle);
                let mut contents = stmt.stmt.lock().unwrap();
                // Actually reading this value would make ASAN fail, but this
                // is what the ODBC standard expects.
                contents.state = StatementState::Unallocated;
                (*contents.connection)
                    .connection
                    .write()
                    .unwrap()
                    .statements
                    .remove(&handle);
            }
            HandleType::Desc => {
                let desc = Box::from_raw(handle as *mut DescriptorHandle);
                // Actually reading this value would make ASAN fail, but this
                // is what the ODBC standard expects.
                desc.descriptor.write().unwrap().state = DescriptorState::Unallocated;
            }
        }
    }
    SqlReturn::SUCCESS
}

#[no_mangle]
pub extern "C" fn SQLFreeStmt(_statement_handle: HStmt, _option: SmallInt) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetConnectAttr(
    _connection_handle: HDbc,
    _attribute: ConnectionAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
    _cursor_name: *mut Char,
    _buffer_length: SmallInt,
    _name_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
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
    _handle: Handle,
    _rec_number: SmallInt,
    _state: *mut Char,
    _native_error_ptr: *mut Integer,
    _message_text: *mut Char,
    _buffer_length: SmallInt,
    _text_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetDiagRecW(
    _handle_type: HandleType,
    _handle: Handle,
    _record_rumber: SmallInt,
    _state: *mut WChar,
    _native_error_ptr: *mut Integer,
    _message_text: *mut WChar,
    _buffer_length: SmallInt,
    _text_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLGetEnvAttr(
    _environment_handle: HEnv,
    _attribute: EnvironmentAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length: *mut Integer,
) -> SqlReturn {
    unimplemented!()
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
    _connection_handle: HDbc,
    _info_type: InfoType,
    _info_value_ptr: Pointer,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
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
    _handle: HStmt,
    _attribute: StatementAttribute,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unimplemented!()
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
    _connection_handle: HDbc,
    _in_statement_text: *const Char,
    _in_statement_len: Integer,
    _out_statement_text: *mut Char,
    _buffer_len: Integer,
    _out_statement_len: *mut Integer,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
    _param_count_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLNumResultCols(
    _statement_handle: HStmt,
    _column_count_ptr: *mut SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLParamData(_hstmt: HStmt, _value_ptr_ptr: *mut Pointer) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLPrepare(
    _hstmt: HStmt,
    _statement_text: *const Char,
    _text_length: Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLPrepareW(
    _hstmt: HStmt,
    _statement_text: *const WChar,
    _text_length: Integer,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLPrimaryKeys(
    _statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _table_name: *const Char,
    _table_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _proc_name: *const Char,
    _proc_name_length: SmallInt,
    _column_name: *const Char,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLProcedureColumnsW(
    _statement_handle: HStmt,
    _catalog_name: *const WChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WChar,
    _schema_name_length: SmallInt,
    _proc_name: *const WChar,
    _proc_name_length: SmallInt,
    _column_name: *const WChar,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLProcedures(
    _statement_handle: HStmt,
    _catalog_name: *const Char,
    _catalog_name_length: SmallInt,
    _schema_name: *const Char,
    _schema_name_length: SmallInt,
    _proc_name: *const Char,
    _proc_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLProceduresW(
    _statement_handle: HStmt,
    _catalog_name: *const WChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WChar,
    _schema_name_length: SmallInt,
    _proc_name: *const WChar,
    _proc_name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLPutData(
    _statement_handle: HStmt,
    _data_ptr: Pointer,
    _str_len_or_ind_ptr: Len,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLRowCount(_statement_handle: HStmt, _row_count_ptr: *mut Len) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetConnectAttr(
    _hdbc: HDbc,
    _attr: ConnectionAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
    _cursor_name: *const Char,
    _name_length: SmallInt,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
    _row_number: ULen,
    _operation: USmallInt,
    _lock_type: USmallInt,
) -> SqlReturn {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn SQLSetEnvAttr(
    _environment_handle: HEnv,
    _attribute: EnvironmentAttribute,
    _value: Pointer,
    _string_length: Integer,
) -> SqlReturn {
    unimplemented!()
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
    _hstmt: HStmt,
    _attr: StatementAttribute,
    _value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
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
    unimplemented!()
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
    _statement_handle: HStmt,
    _catalog_name: *const Char,
    _name_length_1: SmallInt,
    _schema_name: *const Char,
    _name_length_2: SmallInt,
    _table_name: *const Char,
    _name_length_3: SmallInt,
) -> SqlReturn {
    unimplemented!()
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
    _statement_handle: HStmt,
    _catalog_name: *const Char,
    _name_length_1: SmallInt,
    _schema_name: *const Char,
    _name_length_2: SmallInt,
    _table_name: *const Char,
    _name_length_3: SmallInt,
    _table_type: *const Char,
    _name_length_4: SmallInt,
) -> SqlReturn {
    unimplemented!()
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

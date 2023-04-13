use crate::api::{definitions::*, errors::ODBCError};
use logger::Logger;
use odbc_sys::{HDbc, HDesc, HEnv, HStmt, Handle, Len, Pointer, ULen, USmallInt};
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::{HashMap, HashSet},
    ptr::null_mut,
    sync::RwLock,
};

#[derive(Debug)]
pub enum MongoHandle {
    Env(Env),
    Connection(Connection),
    Statement(Statement),
    Descriptor(Descriptor),
}

impl MongoHandle {
    pub fn as_env(&self) -> Option<&Env> {
        match self {
            MongoHandle::Env(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_connection(&self) -> Option<&Connection> {
        match self {
            MongoHandle::Connection(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_statement(&self) -> Option<&Statement> {
        match self {
            MongoHandle::Statement(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_descriptor(&self) -> Option<&Descriptor> {
        match self {
            MongoHandle::Descriptor(d) => Some(d),
            _ => None,
        }
    }

    /// add_diag_info appends a new ODBCError object to the `errors` field.
    pub fn add_diag_info(&mut self, error: ODBCError) {
        match self {
            MongoHandle::Env(e) => {
                e.errors.write().unwrap().push(error);
            }
            MongoHandle::Connection(c) => {
                c.errors.write().unwrap().push(error);
            }
            MongoHandle::Statement(s) => {
                s.errors.write().unwrap().push(error);
            }
            MongoHandle::Descriptor(d) => {
                d.errors.write().unwrap().push(error);
            }
        }
    }

    pub fn clear_diagnostics(&mut self) {
        match self {
            MongoHandle::Env(e) => {
                e.errors.write().unwrap().clear();
            }
            MongoHandle::Connection(c) => {
                c.errors.write().unwrap().clear();
            }
            MongoHandle::Statement(s) => {
                s.errors.write().unwrap().clear();
            }
            MongoHandle::Descriptor(d) => {
                d.errors.write().unwrap().clear();
            }
        }
    }

    ///
    /// Generate a String containing the current handle and its parents address.
    ///
    pub(crate) unsafe fn get_handle_info(&mut self) -> String {
        let mut handle_info = String::new();
        let mut handle = self;
        loop {
            let handle_ptr: *mut MongoHandle = handle;
            match handle {
                MongoHandle::Env(_) => {
                    handle_info = format!("[Env_{handle_ptr:?}]{handle_info}");
                    return handle_info;
                }
                MongoHandle::Connection(c) => {
                    let env = c.env;
                    handle_info = format!("[Conn_{handle_ptr:?}]{handle_info}");
                    if env.is_null() {
                        return handle_info;
                    }
                    handle = &mut *env;
                }
                MongoHandle::Statement(s) => {
                    let conn = s.connection;
                    handle_info = format!("[Stmt_{handle_ptr:?}]{handle_info}");
                    if conn.is_null() {
                        return handle_info;
                    }
                    handle = &mut *conn;
                }
                MongoHandle::Descriptor(d) => {
                    let conn = d.connection;
                    handle_info = format!("[Desc_{handle_ptr:?}]{handle_info}");
                    if conn.is_null() {
                        return handle_info;
                    }
                    handle = &mut *conn;
                }
            }
        }
    }
}

pub type MongoHandleRef = &'static mut MongoHandle;

impl From<Handle> for MongoHandleRef {
    fn from(handle: Handle) -> Self {
        unsafe { (*(handle as *mut MongoHandle)).borrow_mut() }
    }
}

impl From<HEnv> for MongoHandleRef {
    fn from(handle: HEnv) -> Self {
        unsafe { (*(handle as *mut MongoHandle)).borrow_mut() }
    }
}

impl From<HStmt> for MongoHandleRef {
    fn from(handle: HStmt) -> Self {
        unsafe { (*(handle as *mut MongoHandle)).borrow_mut() }
    }
}

impl From<HDbc> for MongoHandleRef {
    fn from(handle: HDbc) -> Self {
        unsafe { (*(handle as *mut MongoHandle)).borrow_mut() }
    }
}

impl From<HDesc> for MongoHandleRef {
    fn from(handle: HDesc) -> Self {
        unsafe { (*(handle as *mut MongoHandle)).borrow_mut() }
    }
}

#[derive(Debug)]
pub struct Env {
    // attributes for this Env. We box the attributes so that the MongoHandle type
    // remains fairly small regardless of underlying handle type.
    pub attributes: RwLock<EnvAttributes>,
    // state of this Env
    pub state: RwLock<EnvState>,
    pub connections: RwLock<HashSet<*mut MongoHandle>>,
    pub errors: RwLock<Vec<ODBCError>>,
    // we need to hold on to the logger so that it doesn't get dropped
    pub logger: RefCell<Option<Logger>>,
}

impl Env {
    pub fn with_state(state: EnvState, logger: RefCell<Option<Logger>>) -> Self {
        Self {
            attributes: RwLock::new(EnvAttributes::default()),
            state: RwLock::new(state),
            connections: RwLock::new(HashSet::new()),
            errors: RwLock::new(vec![]),
            logger,
        }
    }
}

#[derive(Debug)]
pub struct EnvAttributes {
    pub odbc_ver: OdbcVersion,
    pub output_nts: SqlBool,
    pub connection_pooling: ConnectionPooling,
    pub cp_match: CpMatch,
    pub driver_unicode_type: CharSet,
}

impl Default for EnvAttributes {
    fn default() -> Self {
        Self {
            odbc_ver: OdbcVersion::Odbc3_80,
            output_nts: SqlBool::True,
            connection_pooling: ConnectionPooling::Off,
            cp_match: CpMatch::Strict,
            driver_unicode_type: CharSet::Utf16,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EnvState {
    Allocated,
    ConnectionAllocated,
}

#[derive(Debug)]
pub struct Connection {
    // type of this handle for runtime checking purposes.
    // Pointer to the Env from which
    // this Connection was allocated
    pub env: *mut MongoHandle,
    // mongo_connection is the actual connection to the mongo server
    // it will be None when the Connection is closed.
    pub mongo_connection: RwLock<Option<mongo_odbc_core::MongoConnection>>,
    // all the possible Connection settings
    pub attributes: RwLock<ConnectionAttributes>,
    // state of this connection
    pub state: RwLock<ConnectionState>,
    // MongoDB Client for issuing commands
    // pub client: Option<MongoClient>,
    // all Statements allocated from this Connection
    pub statements: RwLock<HashSet<*mut MongoHandle>>,
    pub errors: RwLock<Vec<ODBCError>>,
}

#[derive(Debug, Default)]
pub struct ConnectionAttributes {
    // SQL_ATTR_CURRENT_CATALOG: the current catalog/database
    // for this Connection.
    pub current_catalog: Option<String>,
    // SQL_ATTR_LOGIN_TIMEOUT: SQLUINTEGER, timeout in seconds
    // to wait for a login request to complete.
    pub login_timeout: Option<u32>,
    // SQL_ATTR_CONNECTION_TIMEOUT: SQLUINTER, timeout in seconds
    // to wait for any operation on a connection to timeout (other than
    // initial login).
    pub connection_timeout: Option<u32>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Allocated,
    _ConnectionFunctionNeedsDataEnv,
    Connected,
    StatementAllocated,
    _TransactionInProgress,
}

impl Connection {
    pub fn with_state(env: *mut MongoHandle, state: ConnectionState) -> Self {
        Self {
            env,
            mongo_connection: RwLock::new(None),
            attributes: RwLock::new(ConnectionAttributes::default()),
            state: RwLock::new(state),
            statements: RwLock::new(HashSet::new()),
            errors: RwLock::new(vec![]),
        }
    }
}

#[derive(Debug)]
pub enum CachedData {
    // we do not need an index into fixed data. Attempting to stream fixed data always fails.
    Fixed,
    Char(usize, Vec<u8>),
    Bin(usize, Vec<u8>),
    WChar(usize, Vec<u16>),
}

#[derive(Debug)]
pub struct Statement {
    pub connection: *mut MongoHandle,
    pub mongo_statement: RwLock<Option<Box<dyn mongo_odbc_core::MongoStatement>>>,
    pub var_data_cache: RwLock<Option<HashMap<USmallInt, CachedData>>>,
    pub attributes: RwLock<StatementAttributes>,
    pub state: RwLock<StatementState>,
    // pub cursor: RwLock<Option<Box<Peekable<Cursor>>>>,
    pub errors: RwLock<Vec<ODBCError>>,
}

#[derive(Debug)]
pub struct StatementAttributes {
    pub app_row_desc: *mut MongoHandle,
    pub app_param_desc: *mut MongoHandle,
    pub async_enable: AsyncEnable,
    pub async_stmt_event: Pointer,
    pub cursor_scrollable: CursorScrollable,
    pub cursor_sensitivity: CursorSensitivity,
    pub concurrency: Concurrency,
    pub cursor_type: CursorType,
    pub enable_auto_ipd: SqlBool,
    pub fetch_bookmark_ptr: *mut Len,
    pub imp_row_desc: *mut MongoHandle,
    pub imp_param_desc: *mut MongoHandle,
    pub max_length: ULen,
    pub max_rows: ULen,
    pub no_scan: NoScan,
    pub param_bind_offset_ptr: *mut ULen,
    pub param_bind_type: ULen,
    pub param_operation_ptr: *mut USmallInt,
    pub param_processed_ptr: *mut ULen,
    pub param_status_ptr: *mut USmallInt,
    pub paramset_size: ULen,
    pub query_timeout: ULen,
    pub retrieve_data: RetrieveData,
    pub row_array_size: ULen,
    pub row_bind_offset_ptr: *mut ULen,
    pub row_bind_type: ULen,
    pub row_index_is_valid: bool,
    pub row_number: ULen,
    pub row_operation_ptr: *mut USmallInt,
    pub row_status_ptr: *mut USmallInt,
    pub rows_fetched_ptr: *mut ULen,
    pub simulate_cursor: ULen,
    pub use_bookmarks: UseBookmarks,
}

impl Drop for StatementAttributes {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.app_row_desc);
            let _ = Box::from_raw(self.app_param_desc);
            let _ = Box::from_raw(self.imp_row_desc);
            let _ = Box::from_raw(self.imp_param_desc);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StatementState {
    Allocated,
    _Prepared,
    _PreparedHasResultSet,
    _ExecutedNoResultSet,
    _ExecutedHasResultSet,
    _CursorFetchSet,
    _CursorExtendedFetchSet,
    _FunctionNeedsDataNoParam,
    _FunctionNeedsDataNoPut,
    _FunctionNeedsDataPutCalled,
    _Executing,
    _AsyncCancelled,
}

impl Statement {
    pub fn with_state(connection: *mut MongoHandle, state: StatementState) -> Self {
        let implicit_app_row_desc =
            Descriptor::with_state(connection, DescriptorState::ImplicitlyAllocated);

        let implicit_param_row_desc =
            Descriptor::with_state(connection, DescriptorState::ImplicitlyAllocated);

        let implicit_app_imp_desc =
            Descriptor::with_state(connection, DescriptorState::ImplicitlyAllocated);

        let implicit_param_imp_desc =
            Descriptor::with_state(connection, DescriptorState::ImplicitlyAllocated);

        Self {
            connection,
            state: RwLock::new(state),
            var_data_cache: RwLock::new(None),
            attributes: RwLock::new(StatementAttributes {
                app_row_desc: Box::into_raw(Box::new(MongoHandle::Descriptor(
                    implicit_app_row_desc,
                ))),
                app_param_desc: Box::into_raw(Box::new(MongoHandle::Descriptor(
                    implicit_param_row_desc,
                ))),
                async_enable: AsyncEnable::Off,
                async_stmt_event: null_mut(),
                cursor_scrollable: CursorScrollable::NonScrollable,
                cursor_sensitivity: CursorSensitivity::Insensitive,
                concurrency: Concurrency::ReadOnly,
                cursor_type: CursorType::ForwardOnly,
                enable_auto_ipd: SqlBool::False,
                fetch_bookmark_ptr: null_mut(),
                imp_row_desc: Box::into_raw(Box::new(MongoHandle::Descriptor(
                    implicit_app_imp_desc,
                ))),
                imp_param_desc: Box::into_raw(Box::new(MongoHandle::Descriptor(
                    implicit_param_imp_desc,
                ))),
                max_length: 0,
                max_rows: 0,
                no_scan: NoScan::Off,
                param_bind_offset_ptr: null_mut(),
                param_bind_type: BindType::BindByColumn as usize,
                param_operation_ptr: null_mut(),
                param_processed_ptr: null_mut(),
                param_status_ptr: null_mut(),
                paramset_size: 0,
                query_timeout: 0,
                retrieve_data: RetrieveData::Off,
                row_array_size: 1,
                row_bind_offset_ptr: null_mut(),
                row_bind_type: BindType::BindByColumn as usize,
                row_index_is_valid: false,
                row_number: 0,
                row_operation_ptr: null_mut(),
                row_status_ptr: null_mut(),
                rows_fetched_ptr: null_mut(),
                simulate_cursor: SimulateCursor::NonUnique as usize,
                use_bookmarks: UseBookmarks::Off,
            }),
            errors: RwLock::new(vec![]),
            mongo_statement: RwLock::new(None),
        }
    }

    pub(crate) fn insert_var_data_cache(&self, col: u16, data: CachedData) {
        self.var_data_cache
            .write()
            .unwrap()
            .as_mut()
            .unwrap()
            .insert(col, data);
    }
}

#[derive(Debug)]
pub struct Descriptor {
    pub connection: *mut MongoHandle,
    pub attributes: RwLock<DescriptorAttributes>,
    pub state: RwLock<DescriptorState>,
    pub errors: RwLock<Vec<ODBCError>>,
}

/// See https://learn.microsoft.com/en-us/sql/odbc/reference/appendixes/descriptor-transitions for
/// states and transitions
#[derive(Debug, PartialEq, Eq)]
pub enum DescriptorState {
    ImplicitlyAllocated, // D1i
    ExplicitlyAllocated, // D1e
}

#[derive(Debug, Default)]
pub struct DescriptorAttributes {}

impl Descriptor {
    pub fn with_state(connection: *mut MongoHandle, state: DescriptorState) -> Self {
        Self {
            connection,
            attributes: RwLock::new(DescriptorAttributes::default()),
            state: RwLock::new(state),
            errors: RwLock::new(vec![]),
        }
    }
}

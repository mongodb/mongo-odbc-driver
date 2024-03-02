use crate::api::errors::ODBCError;
use bson::{Bson, Uuid};
use cstr::{Charset, WideChar};
use definitions::{
    AsyncEnable, AttrConnectionPooling, AttrCpMatch, AttrOdbcVersion, BindType, Concurrency,
    CursorScrollable, CursorSensitivity, CursorType, HDbc, HDesc, HEnv, HStmt, Handle, Len, NoScan,
    Pointer, RetrieveData, SimulateCursor, SmallInt, SqlBool, ULen, USmallInt, UseBookmarks,
};
use mongo_odbc_core::TypeMode;
use std::{
    borrow::BorrowMut,
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

    /// Returns a reference to the statement's connection, if there is
    /// one.
    pub fn as_statement_connection(&self) -> Option<&Connection> {
        match self {
            MongoHandle::Statement(stmt) => unsafe {
                stmt.connection.as_ref().unwrap().as_connection()
            },
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

    /// get the odbc_version from the underlying env handle, used to handle
    /// behavior that is different between odbc versions properly
    pub fn get_odbc_version(&mut self) -> AttrOdbcVersion {
        let env = match self {
            MongoHandle::Env(_) => self,
            MongoHandle::Connection(conn) => conn.env,
            MongoHandle::Descriptor(Descriptor {
                connection: conn, ..
            })
            | MongoHandle::Statement(Statement {
                connection: conn, ..
            }) => unsafe { conn.as_ref().unwrap().as_connection().unwrap().env },
        };
        unsafe {
            env.as_ref()
                .unwrap()
                .as_env()
                .unwrap()
                .attributes
                .read()
                .unwrap()
                .odbc_ver
        }
    }
}

#[macro_export]
/// A utility macro that returns a boolean on whether the handle exhibits odbc 3 behavior or not
macro_rules! has_odbc_3_behavior {
    ($handle:expr) => {{
        match (*$handle).get_odbc_version() {
            AttrOdbcVersion::SQL_OV_ODBC2 => false,
            AttrOdbcVersion::SQL_OV_ODBC3 | AttrOdbcVersion::SQL_OV_ODBC3_80 => true,
        }
    }};
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
}

impl Env {
    pub fn with_state(state: EnvState) -> Self {
        Self {
            attributes: RwLock::new(EnvAttributes::default()),
            state: RwLock::new(state),
            connections: RwLock::new(HashSet::new()),
            errors: RwLock::new(vec![]),
        }
    }
}

#[derive(Debug)]
pub struct EnvAttributes {
    pub odbc_ver: AttrOdbcVersion,
    pub output_nts: SqlBool,
    pub connection_pooling: AttrConnectionPooling,
    pub cp_match: AttrCpMatch,
    pub driver_unicode_type: Charset,
}

impl Default for EnvAttributes {
    fn default() -> Self {
        Self {
            odbc_ver: AttrOdbcVersion::SQL_OV_ODBC3_80,
            output_nts: SqlBool::SQL_TRUE,
            connection_pooling: AttrConnectionPooling::SQL_CP_OFF,
            cp_match: AttrCpMatch::SQL_CP_STRICT_MATCH,
            driver_unicode_type: cstr::CHARSET,
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
    // type_mode indicates if BsonTypeInfo.simple_type_info will be
    // utilized in place of standard BsonTypeInfo fields
    pub type_mode: RwLock<TypeMode>,
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
            type_mode: RwLock::new(TypeMode::Standard),
        }
    }
}

#[derive(Debug)]
pub enum CachedData {
    // we do not need an index into fixed data. Attempting to stream fixed data always fails.
    Fixed,
    Char(usize, Vec<u8>),
    Bin(usize, Vec<u8>),
    WChar(usize, Vec<WideChar>),
}

#[derive(Debug)]
pub struct Statement {
    pub connection: *mut MongoHandle,
    pub mongo_statement: RwLock<Option<Box<dyn mongo_odbc_core::MongoStatement>>>,
    pub var_data_cache: RwLock<Option<HashMap<USmallInt, CachedData>>>,
    pub attributes: RwLock<StatementAttributes>,
    pub state: RwLock<StatementState>,
    pub statement_id: RwLock<Bson>,
    // pub cursor: RwLock<Option<Box<Peekable<Cursor>>>>,
    pub errors: RwLock<Vec<ODBCError>>,
    pub bound_cols: RwLock<Option<HashMap<USmallInt, BoundColInfo>>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BoundColInfo {
    pub target_type: SmallInt,
    pub target_buffer: Pointer,
    pub buffer_length: Len,
    pub length_or_indicator: *mut Len,
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
    SynchronousQueryExecuting,
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
            statement_id: RwLock::new(Uuid::new().into()),
            var_data_cache: RwLock::new(None),
            attributes: RwLock::new(StatementAttributes {
                app_row_desc: Box::into_raw(Box::new(MongoHandle::Descriptor(
                    implicit_app_row_desc,
                ))),
                app_param_desc: Box::into_raw(Box::new(MongoHandle::Descriptor(
                    implicit_param_row_desc,
                ))),
                async_enable: AsyncEnable::SQL_ASYNC_ENABLE_OFF,
                async_stmt_event: null_mut(),
                cursor_scrollable: CursorScrollable::SQL_NONSCROLLABLE,
                cursor_sensitivity: CursorSensitivity::SQL_INSENSITIVE,
                concurrency: Concurrency::SQL_CONCUR_READ_ONLY,
                cursor_type: CursorType::SQL_CURSOR_FORWARD_ONLY,
                enable_auto_ipd: SqlBool::SQL_FALSE,
                fetch_bookmark_ptr: null_mut(),
                imp_row_desc: Box::into_raw(Box::new(MongoHandle::Descriptor(
                    implicit_app_imp_desc,
                ))),
                imp_param_desc: Box::into_raw(Box::new(MongoHandle::Descriptor(
                    implicit_param_imp_desc,
                ))),
                max_length: 0,
                max_rows: 0,
                no_scan: NoScan::SQL_NOSCAN_OFF,
                param_bind_offset_ptr: null_mut(),
                param_bind_type: BindType::SQL_BIND_BY_COLUMN as usize,
                param_operation_ptr: null_mut(),
                param_processed_ptr: null_mut(),
                param_status_ptr: null_mut(),
                paramset_size: 0,
                query_timeout: 0,
                retrieve_data: RetrieveData::Off,
                row_array_size: 1,
                row_bind_offset_ptr: null_mut(),
                row_bind_type: BindType::SQL_BIND_BY_COLUMN as usize,
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
            bound_cols: RwLock::new(None),
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

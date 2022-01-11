use crate::errors::ODBCError;
use odbc_sys::{HDbc, HEnv, HStmt, Handle, Integer};
use std::{borrow::BorrowMut, collections::HashSet, sync::RwLock};

#[derive(Debug)]
pub enum MongoHandle {
    Env(RwLock<Env>),
    Connection(RwLock<Connection>),
    Statement(RwLock<Statement>),
}

impl MongoHandle {
    pub fn as_env(&self) -> Option<&RwLock<Env>> {
        match self {
            MongoHandle::Env(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_connection(&self) -> Option<&RwLock<Connection>> {
        match self {
            MongoHandle::Connection(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_statement(&self) -> Option<&RwLock<Statement>> {
        match self {
            MongoHandle::Statement(s) => Some(s),
            _ => None,
        }
    }

    /// add_diag_info appends a new ODBCError object to the `errors` field.
    pub fn add_diag_info(&mut self, error: ODBCError) {
        match self {
            MongoHandle::Env(e) => {
                let mut env_contents = (*e).write().unwrap();
                env_contents.errors.push(error);
            }
            MongoHandle::Connection(c) => {
                let mut dbc_contents = (*c).write().unwrap();
                dbc_contents.errors.push(error);
            }
            MongoHandle::Statement(s) => {
                let mut stmt_contents = (*s).write().unwrap();
                stmt_contents.errors.push(error);
            }
        }
    }

    pub fn clear_diagnostics(&mut self) {
        match self {
            MongoHandle::Env(e) => {
                let mut env_contents = (*e).write().unwrap();
                env_contents.errors.clear();
            }
            MongoHandle::Connection(c) => {
                let mut dbc_contents = (*c).write().unwrap();
                dbc_contents.errors.clear();
            }
            MongoHandle::Statement(s) => {
                let mut stmt_contents = (*s).write().unwrap();
                stmt_contents.errors.clear();
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

#[derive(Debug)]
pub struct Env {
    // attributes for this Env. We box the attributes so that the MongoHandle type
    // remains fairly small regardless of underlying handle type.
    pub _attributes: Box<EnvAttributes>,
    // state of this Env
    pub state: EnvState,
    pub connections: HashSet<*mut MongoHandle>,
    pub errors: Vec<ODBCError>,
}

impl Env {
    pub fn with_state(state: EnvState) -> Self {
        Self {
            _attributes: Box::new(EnvAttributes::default()),
            state,
            connections: HashSet::new(),
            errors: vec![],
        }
    }
}

#[derive(Debug)]
pub struct EnvAttributes {
    pub odbc_ver: Integer,
}

impl Default for EnvAttributes {
    fn default() -> Self {
        Self { odbc_ver: 3 }
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
    // all the possible Connection settings
    pub _attributes: Box<ConnectionAttributes>,
    // state of this connection
    pub state: ConnectionState,
    // MongoDB Client for issuing commands
    // pub client: Option<MongoClient>,
    // all Statements allocated from this Connection
    pub statements: HashSet<*mut MongoHandle>,
    pub errors: Vec<ODBCError>,
}

#[derive(Debug, Default)]
pub struct ConnectionAttributes {
    pub current_db: Option<String>,
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
            _attributes: Box::new(ConnectionAttributes::default()),
            state,
            statements: HashSet::new(),
            errors: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Statement {
    pub connection: *mut MongoHandle,
    pub _attributes: Box<StatementAttributes>,
    pub state: StatementState,
    //pub cursor: Option<Box<Peekable<Cursor>>>,
    pub errors: Vec<ODBCError>,
}

#[derive(Debug, Default)]
pub struct StatementAttributes {}

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
        Self {
            connection,
            _attributes: Box::new(StatementAttributes::default()),
            state,
            errors: vec![],
        }
    }
}

use std::{collections::HashSet, sync::RwLock};

use odbc_sys::Integer;

#[derive(Debug)]
pub enum MongoHandle {
    Env(RwLock<Env>),
    Connection(RwLock<Connection>),
    Statement(RwLock<Statement>),
    #[allow(dead_code)]
    Descriptor(RwLock<Descriptor>),
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

    pub fn as_descriptor(&self) -> Option<&RwLock<Descriptor>> {
        match self {
            MongoHandle::Descriptor(d) => Some(d),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ODBCError {
    pub sql_state: String,
    pub error_message: String,
    pub native_err_code: i32,
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

#[derive(Debug)]
pub struct Descriptor {
    pub errors: Vec<ODBCError>,
}

impl Descriptor {
    #[allow(dead_code)]
    pub fn default() -> Descriptor {
        Self { errors: vec![] }
    }
}

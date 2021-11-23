use std::{collections::HashSet, sync::RwLock};

use odbc_sys::Integer;

#[derive(Debug)]
pub enum MongoHandle {
    Env(RwLock<Env>),
    Connection(RwLock<Connection>),
    Statement(RwLock<Statement>),
}

impl MongoHandle {
    pub fn as_env(&self) -> Result<&RwLock<Env>, ()> {
        match self {
            MongoHandle::Env(e) => Ok(e),
            _ => Err(()),
        }
    }

    pub fn as_connection(&self) -> Result<&RwLock<Connection>, ()> {
        match self {
            MongoHandle::Connection(c) => Ok(c),
            _ => Err(()),
        }
    }

    pub fn as_statement(&self) -> Result<&RwLock<Statement>, ()> {
        match self {
            MongoHandle::Statement(s) => Ok(s),
            _ => Err(()),
        }
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
}

impl Env {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            _attributes: Box::new(EnvAttributes::default()),
            state: EnvState::Unallocated,
            connections: HashSet::new(),
        }
    }

    pub fn with_state(state: EnvState) -> Self {
        Self {
            _attributes: Box::new(EnvAttributes::default()),
            state,
            connections: HashSet::new(),
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
    Unallocated,
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
    // all Descriptors attached to this Connection
    pub _descriptors: HashSet<*mut MongoHandle>,
    // all Statements allocated from this Connection
    pub statements: HashSet<*mut MongoHandle>,
}

#[derive(Debug)]
pub struct ConnectionAttributes {
    pub current_db: Option<String>,
}

impl Default for ConnectionAttributes {
    fn default() -> Self {
        Self { current_db: None }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionState {
    _UnallocatedEnvUnallocatedConnection,
    AllocatedEnvUnallocatedConnection,
    AllocatedEnvAllocatedConnection,
    _ConnectionFunctionNeedsDataEnv,
    Connected,
    StatementAllocated,
    _TransactionInProgress,
}

impl Connection {
    #[cfg(test)]
    pub fn new(env: *mut MongoHandle) -> Self {
        Self {
            env,
            _attributes: Box::new(ConnectionAttributes::default()),
            state: ConnectionState::AllocatedEnvUnallocatedConnection,
            _descriptors: HashSet::new(),
            statements: HashSet::new(),
        }
    }

    pub fn with_state(env: *mut MongoHandle, state: ConnectionState) -> Self {
        Self {
            env,
            _attributes: Box::new(ConnectionAttributes::default()),
            state,
            _descriptors: HashSet::new(),
            statements: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub struct Statement {
    pub connection: *mut MongoHandle,
    pub _attributes: Box<StatementAttributes>,
    pub state: StatementState,
    //pub cursor: Option<Box<Peekable<Cursor>>>,
}

#[derive(Debug)]
pub struct StatementAttributes {}

impl Default for StatementAttributes {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StatementState {
    Unallocated,
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
    #[cfg(test)]
    pub fn new(connection: *mut MongoHandle) -> Self {
        Self {
            connection,
            _attributes: Box::new(StatementAttributes::default()),
            state: StatementState::Unallocated,
        }
    }

    pub fn with_state(connection: *mut MongoHandle, state: StatementState) -> Self {
        Self {
            connection,
            _attributes: Box::new(StatementAttributes::default()),
            state,
        }
    }
}

use std::{
    collections::HashSet,
    sync::{Mutex, RwLock},
};

use odbc_sys::Integer;

#[derive(Debug)]
pub struct EnvHandle {
    pub env: RwLock<Env>,
}

impl EnvHandle {
    pub fn new() -> Self {
        Self {
            env: RwLock::new(Env {
                _attributes: EnvAttributes::default(),
                state: EnvState::Unallocated,
                connections: HashSet::new(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct Env {
    // attributes for this Env
    pub _attributes: EnvAttributes,
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
pub enum EnvState {
    Unallocated,
    Allocated,
}

#[derive(Debug)]
pub struct ConnectionHandle {
    pub connection: RwLock<Connection>,
}

#[derive(Debug)]
pub struct Connection {
    // Pointer to the Env from which
    // this Connection was allocated
    pub env: *mut EnvHandle,
    // all the possible Connection settings
    pub _attributes: ConnectionAttributes,
    // state of this connection
    pub state: ConnectionState,
    // MongoDB Client for issuing commands
    // pub client: Option<MongoClient>,
    // all Descriptors attached to this Connection
    pub _descriptors: HashSet<*mut DescriptorHandle>,
    // all Statements allocated from this Connection
    pub statements: HashSet<*mut StatementHandle>,
}

#[derive(Debug)]
pub struct ConnectionAttributes {}

impl Default for ConnectionAttributes {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionState {
    _UnallocatedEnvUnallocatedConnection,
    AllocatedEnvUnallocatedConnection,
    AllocatedEnvAllocatedConnection,
    _ConnectionFunctionNeedsDataEnv,
    _Connected,
    _StatementAllocated,
    _TransactionInProgress,
}

impl ConnectionHandle {
    pub fn new(env: *mut EnvHandle) -> Self {
        Self {
            connection: RwLock::new(Connection::new(env)),
        }
    }
}

impl Connection {
    pub fn new(env: *mut EnvHandle) -> Self {
        Self {
            env,
            _attributes: ConnectionAttributes::default(),
            state: ConnectionState::AllocatedEnvUnallocatedConnection,
            _descriptors: HashSet::new(),
            statements: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub struct StatementHandle {
    pub stmt: Mutex<Statement>,
}

#[derive(Debug)]
pub struct Statement {
    pub connection: *mut ConnectionHandle,
    pub _attributes: StatementAttributes,
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

impl StatementHandle {
    pub fn new(connection: *mut ConnectionHandle) -> Self {
        Self {
            stmt: Mutex::new(Statement::new(connection)),
        }
    }
}

impl Statement {
    pub fn new(connection: *mut ConnectionHandle) -> Self {
        Self {
            connection,
            _attributes: StatementAttributes::default(),
            state: StatementState::Unallocated,
        }
    }
}

#[derive(Debug)]
pub struct DescriptorHandle {
    pub descriptor: RwLock<Descriptor>,
}

impl DescriptorHandle {
    pub fn new() -> Self {
        Self {
            descriptor: RwLock::new(Descriptor::new()),
        }
    }
}

#[derive(Debug)]
pub struct Descriptor {
    pub state: DescriptorState,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DescriptorState {
    Unallocated,
    _ImplicitlyAllocated,
    ExplicitlyAllocated,
}

impl Descriptor {
    pub fn new() -> Self {
        Self {
            state: DescriptorState::Unallocated,
        }
    }
}

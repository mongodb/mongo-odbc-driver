use std::{collections::HashSet, sync::RwLock};

use odbc_sys::{HandleType, Integer};

pub type EnvHandle = RwLock<Env>;

// repr(C) is required so that all handle structs have handle_type in the same offset.
#[derive(Debug)]
#[repr(C)]
pub struct Env {
    pub handle_type: HandleType,
    // attributes for this Env
    pub _attributes: EnvAttributes,
    // state of this Env
    pub state: EnvState,
    pub connections: HashSet<*mut ConnectionHandle>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            handle_type: HandleType::Env,
            _attributes: EnvAttributes::default(),
            state: EnvState::Unallocated,
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

pub type ConnectionHandle = RwLock<Connection>;

// repr(C) is required so that all handle structs have handle_type in the same offset.
#[derive(Debug)]
#[repr(C)]
pub struct Connection {
    // type of this handle for runtime checking purposes.
    pub handle_type: HandleType,
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

impl Connection {
    pub fn new(env: *mut EnvHandle) -> Self {
        Self {
            handle_type: HandleType::Dbc,
            env,
            _attributes: ConnectionAttributes::default(),
            state: ConnectionState::AllocatedEnvUnallocatedConnection,
            _descriptors: HashSet::new(),
            statements: HashSet::new(),
        }
    }
}

pub type StatementHandle = RwLock<Statement>;

// repr(C) is required so that all handle structs have handle_type in the same offset.
#[derive(Debug)]
#[repr(C)]
pub struct Statement {
    pub handle_type: HandleType,
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

impl Statement {
    pub fn new(connection: *mut ConnectionHandle) -> Self {
        Self {
            handle_type: HandleType::Stmt,
            connection,
            _attributes: StatementAttributes::default(),
            state: StatementState::Unallocated,
        }
    }
}

pub type DescriptorHandle = RwLock<Descriptor>;

// repr(C) is required so that all handle structs have handle_type in the same offset.
#[derive(Debug)]
#[repr(C)]
pub struct Descriptor {
    pub handle_type: HandleType,
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
            handle_type: HandleType::Desc,
            state: DescriptorState::Unallocated,
        }
    }
}

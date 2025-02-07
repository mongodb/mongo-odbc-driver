use crate::{
    add_diag_with_function,
    api::{
        data::{i16_len, i32_len, ptr_safe_write},
        diag::{get_diag_fieldw, get_diag_recw, get_stmt_diag_field},
        errors::{ODBCError, Result},
        util::{connection_attribute_to_string, handle_sql_type, statement_attribute_to_string},
    },
    handles::definitions::*,
    has_odbc_3_behavior, trace_odbc,
};
use constants::*;
use mongodb::bson::{doc, Bson};

use cstr::{input_text_to_string_w, input_text_to_string_w_allow_null, Charset, WideChar};

use definitions::{
    AccessMode, AllocType, AsyncEnable, AttrConnectionPooling, AttrCpMatch, AttrOdbcVersion,
    BindType, CDataType, Concurrency, ConnectionAttribute, CursorScrollable, CursorSensitivity,
    CursorType, Desc, DiagType, DriverConnectOption, EnvironmentAttribute, FetchOrientation,
    FreeStmtOption, HDbc, HDesc, HEnv, HStmt, HWnd, Handle, HandleType, Integer, Len, NoScan,
    Pointer, RetCode, RetrieveData, RowStatus, SmallInt, SqlBool, SqlDataType, SqlReturn,
    StatementAttribute, ULen, USmallInt, UseBookmarks, SQL_NTS,
};
use function_name::named;
use log::{debug, error, info};
use logger::Logger;
use mongo_odbc_core::{
    odbc_uri::ODBCUri, Error, MongoColMetadata, MongoCollections, MongoConnection, MongoDatabases,
    MongoFields, MongoForeignKeys, MongoPrimaryKeys, MongoQuery, MongoStatement, MongoTableTypes,
    MongoTypesInfo, TypeMode,
};
use num_traits::FromPrimitive;
use std::ptr::null_mut;
use std::{collections::HashMap, mem::size_of, panic, sync::mpsc};

const NULL_HANDLE_ERROR: &str = "handle cannot be null";
const HANDLE_MUST_BE_ENV_ERROR: &str = "handle must be env";
const HANDLE_MUST_BE_CONN_ERROR: &str = "handle must be conn";
const HANDLE_MUST_BE_STMT_ERROR: &str = "handle must be stmt";
const HANDLE_MUST_BE_DESC_ERROR: &str = "handle must be desc";

///
/// trace_outcome returns a formatted readable sql return type
///
pub fn trace_outcome(sql_return: &SqlReturn) -> String {
    let outcome = match *sql_return {
        SqlReturn::SUCCESS => "SUCCESS",
        SqlReturn::ERROR => "ERROR",
        SqlReturn::SUCCESS_WITH_INFO => "SUCCESS_WITH_INFO",
        SqlReturn::INVALID_HANDLE => "INVALID_HANDLE",
        SqlReturn::NEED_DATA => "NEED_DATA",
        SqlReturn::NO_DATA => "NO_DATA",
        SqlReturn::PARAM_DATA_AVAILABLE => "PARAM_DATA_AVAILABLE",
        SqlReturn::STILL_EXECUTING => "STILL_EXECUTING",
        _ => "unknown sql_return",
    };
    format!("SQLReturn = {outcome}")
}

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

macro_rules! must_be_desc {
    ($handle:expr) => {{
        let desc = (*$handle).as_descriptor();
        must_be_valid!(desc)
    }};
}

macro_rules! odbc_unwrap {
    ($value:expr, $handle:expr) => {{
        // force the expression
        let value = $value;
        if let Err(error) = value {
            let odbc_err: ODBCError = error.into();
            add_diag_info!($handle, odbc_err.clone());
            return SqlReturn::ERROR;
        }
        value.unwrap()
    }};
}

macro_rules! try_mongo_handle {
    ($raw:expr) => {{
        match MongoHandleRef::try_from($raw) {
            Ok(handle) => handle,
            Err(_) => return SqlReturn::INVALID_HANDLE,
        }
    }};
}
pub(crate) use try_mongo_handle;

/// panic_safe_exec_clear_diagnostics executes `function` such that any panics do not crash the runtime,
/// while clearing any diagnostics in the $handle's error vec.
/// If a panic occurs during execution, the panic is caught and turned into a String.
/// The panic message is added to the diagnostics of `handle` and SqlReturn::ERROR returned.
macro_rules! panic_safe_exec_clear_diagnostics {
    ($level:ident, $function:expr, $handle:expr) => {{
        use crate::panic_safe_exec_keep_diagnostics;
        let handle = $handle;
        let handle_ref = try_mongo_handle!(handle);
        handle_ref.clear_diagnostics();
        panic_safe_exec_keep_diagnostics!($level, $function, $handle);
    }};
}
pub(crate) use panic_safe_exec_clear_diagnostics;

/// panic_safe_exec_keep_diagnostics executes `function` such that any panics do not crash the runtime,
/// while retaining any diagnostics in the provided $handle's errors vec.
/// If a panic occurs during execution, the panic is caught and turned into a String.
/// The panic message is added to the diagnostics of `handle` and SqlReturn::ERROR returned.
macro_rules! panic_safe_exec_keep_diagnostics {
    ($level:ident, $function:expr, $handle:expr) => {{
        let function = $function;
        let handle = $handle;
        let mut maybe_handle_ref = MongoHandleRef::try_from(handle).ok();
        let previous_hook = panic::take_hook();
        let (s, r) = mpsc::sync_channel(1);
        let fct_name: &str = function_name!();
        panic::set_hook(Box::new(move |i| {
            if let Some(location) = i.location() {
                let info = format!("in file '{}' at line {}", location.file(), location.line());
                let _ = s.send(info);
            }
        }));
        let result = panic::catch_unwind(function);
        panic::set_hook(previous_hook);
        match result {
            Ok(sql_return) => {
                #[allow(unused_variables)]
                let trace = trace_outcome(&sql_return);
                if let Some(ref mut h) = maybe_handle_ref {
                    crate::trace_odbc!($level, h, trace, fct_name);
                } else {
                    crate::trace_odbc!($level, trace, fct_name);
                }

                return sql_return;
            }
            Err(err) => {
                let panic_msg = if let Some(msg) = err.downcast_ref::<&'static str>() {
                    format!("{}\n{:?}", msg, r.recv())
                } else {
                    format!("{:?}\n{:?}", err, r.recv())
                };
                let sql_return = SqlReturn::ERROR;
                if let Some(ref mut h) = maybe_handle_ref {
                    add_diag_with_function!(h, ODBCError::Panic(panic_msg.clone()), fct_name);
                    crate::trace_odbc_error!(h, trace_outcome(&sql_return), fct_name);
                } else {
                    crate::trace_odbc_error!(ODBCError::Panic(panic_msg.clone()), fct_name);
                    crate::trace_odbc_error!(trace_outcome(&sql_return), fct_name);
                }
                return sql_return;
            }
        };
    }};
}
pub(crate) use panic_safe_exec_keep_diagnostics;
use shared_sql_utils::driver_settings::DriverSettings;

///
/// unsupported_function is a macro for correctly setting the state for unsupported functions.
/// This macro is used for the SQL functions which the driver has no plan to support in the future.
///
macro_rules! unsupported_function {
    ($handle:expr) => {
        panic_safe_exec_clear_diagnostics!(
            info,
            || {
                let mongo_handle = try_mongo_handle!($handle);
                let name = function_name!();
                add_diag_info!(mongo_handle, ODBCError::Unimplemented(name));
                SqlReturn::ERROR
            },
            $handle
        )
    };
}

///
/// unimpl is a macro for correctly handling the error coming from the Rust unimplemented! panic.
/// This macro is used for the SQL functions which we plan to support but did not implement yet.
///
macro_rules! unimpl {
    ($handle:expr) => {
        panic_safe_exec_clear_diagnostics!(error, || { unimplemented!() }, $handle)
    };
}

macro_rules! add_diag_info {
    ($handle:expr, $error:expr) => {
        add_diag_with_function!($handle, $error, function_name!());
    };
}

///
/// [`SQLAllocHandle`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLAllocHandle-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLAllocHandle(
    handle_type: HandleType,
    input_handle: Handle,
    output_handle: *mut Handle,
) -> SqlReturn {
    panic_safe_exec_keep_diagnostics!(
        info,
        || {
            match sql_alloc_handle(handle_type, input_handle.cast(), output_handle) {
                Ok(_) => SqlReturn::SUCCESS,
                Err(_) => SqlReturn::INVALID_HANDLE,
            }
        },
        input_handle
    );
}

fn sql_alloc_handle(
    handle_type: HandleType,
    input_handle: *mut MongoHandle,
    output_handle: *mut Handle,
) -> Result<()> {
    match handle_type {
        HandleType::SQL_HANDLE_ENV => {
            let env = Env::with_state(EnvState::Allocated);

            let mh = Box::new(MongoHandle::Env(env));
            unsafe {
                *output_handle = Box::into_raw(mh).cast();
            }

            // Read the log level from the driver settings and set the logger level accordingly
            let driver_settings: DriverSettings =
                DriverSettings::from_private_profile_string().unwrap_or_default();
            Logger::set_log_level(driver_settings.log_level);

            Ok(())
        }
        HandleType::SQL_HANDLE_DBC => {
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
            let conn = Connection::with_state(input_handle, ConnectionState::Allocated);
            let mh = Box::new(MongoHandle::Connection(conn));
            let mh_ptr = Box::into_raw(mh);
            env.connections.write().unwrap().insert(mh_ptr);
            *(env.state.write().unwrap()) = EnvState::ConnectionAllocated;
            unsafe { *output_handle = mh_ptr.cast() }
            Ok(())
        }
        HandleType::SQL_HANDLE_STMT => {
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
            let stmt = Statement::with_state(input_handle, StatementState::Allocated);
            let mh = Box::new(MongoHandle::Statement(stmt));
            let mh_ptr = Box::into_raw(mh);
            conn.statements.write().unwrap().insert(mh_ptr);
            *(conn.state.write().unwrap()) = ConnectionState::StatementAllocated;
            unsafe { *output_handle = mh_ptr.cast() }
            Ok(())
        }
        HandleType::SQL_HANDLE_DESC => {
            if input_handle.is_null() {
                return Err(ODBCError::InvalidHandleType(NULL_HANDLE_ERROR));
            }
            // input handle must be a Connection
            unsafe {
                (*input_handle)
                    .as_connection()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_CONN_ERROR))?
            };
            let desc = Descriptor::with_state(input_handle, DescriptorState::ExplicitlyAllocated);
            let mh = Box::new(MongoHandle::Descriptor(desc));
            let mh_ptr = Box::into_raw(mh);
            unsafe { *output_handle = mh_ptr.cast() }
            Ok(())
        }
    }
}

///
/// [`SQLBindCol`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLBindCol-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLBindCol(
    hstmt: HStmt,
    col_number: USmallInt,
    target_type: SmallInt,
    target_value: Pointer,
    buffer_length: Len,
    length_or_indicator: *mut Len,
) -> SqlReturn {
    panic_safe_exec_keep_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(hstmt);
            let stmt = must_be_valid!((*mongo_handle).as_statement());

            // Currently, we only support column binding with no offsets.
            // Make sure that column-wise binding is being used.
            if stmt.attributes.read().unwrap().row_bind_type != BindType::SQL_BIND_BY_COLUMN as ULen
            {
                let mongo_handle = try_mongo_handle!(hstmt);
                add_diag_info!(
                    mongo_handle,
                    ODBCError::Unimplemented("`row-wise column binding`")
                );
                return SqlReturn::ERROR;
            }

            // Make sure that offsets are not being used in column binding.
            if !stmt
                .attributes
                .read()
                .unwrap()
                .row_bind_offset_ptr
                .is_null()
            {
                let mongo_handle = try_mongo_handle!(hstmt);
                add_diag_info!(
                    mongo_handle,
                    ODBCError::Unimplemented("`column binding with offsets`")
                );
                return SqlReturn::ERROR;
            }

            // Make sure that a query was executed/prepared and the number of columns for the resultset is known.
            let mongo_stmt = stmt.mongo_statement.read().unwrap();
            if mongo_stmt.is_none() {
                stmt.errors.write().unwrap().push(ODBCError::NoResultSet);
                return SqlReturn::ERROR;
            }

            let max_string_length = stmt.get_max_string_length();

            let max_col_index = mongo_stmt
                .as_ref()
                .unwrap()
                .get_resultset_metadata(max_string_length)
                .len();

            // Make sure that col_number is in bounds. Columns are 1-indexed as per the ODBC spec.
            if (col_number as usize) > max_col_index || col_number == 0 {
                let mongo_handle = try_mongo_handle!(hstmt);
                add_diag_info!(mongo_handle, ODBCError::InvalidColumnNumber(col_number));
                return SqlReturn::ERROR;
            }

            // make sure that target_type is valid.
            if <CDataType as FromPrimitive>::from_i16(target_type).is_none() {
                let mongo_handle = try_mongo_handle!(hstmt);
                add_diag_info!(mongo_handle, ODBCError::InvalidTargetType(target_type));
                return SqlReturn::ERROR;
            }

            if stmt.bound_cols.read().unwrap().is_none() {
                *stmt.bound_cols.write().unwrap() = Some(HashMap::new());
            }

            // Unbind column if target_value is null
            if target_value.is_null() {
                stmt.bound_cols
                    .write()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .remove(&col_number);
            }
            // Bind column or rebind column with a new value
            else {
                let bound_col_info = BoundColInfo {
                    target_type,
                    target_buffer: target_value,
                    buffer_length,
                    length_or_indicator,
                };

                stmt.bound_cols
                    .write()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .insert(col_number, bound_col_info);
            }

            SqlReturn::SUCCESS
        },
        hstmt
    );
}

///
/// [`SQLBindParameter`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLBindParameter-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
//
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLBindParameter(
    hstmt: HStmt,
    _parameter_number: USmallInt,
    _input_output_type: SmallInt,
    _value_type: SmallInt,
    _parmeter_type: SmallInt,
    _column_size: ULen,
    _decimal_digits: SmallInt,
    _parameter_value_ptr: Pointer,
    _buffer_length: Len,
    _str_len_or_ind_ptr: *mut Len,
) -> SqlReturn {
    unsupported_function!(hstmt)
}

///
/// [`SQLBrowseConnectW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLBrowseConnect-function
///
/// This is the WideChar version of the SQLBrowseConnect function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLBrowseConnectW(
    connection_handle: HDbc,
    _in_connection_string: *const WideChar,
    _string_length: SmallInt,
    _out_connection_string: *mut WideChar,
    _buffer_length: SmallInt,
    _out_buffer_length: *mut SmallInt,
) -> SqlReturn {
    unimpl!(connection_handle);
}

///
/// [`SQLBulkOperations`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLBulkOperations-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLBulkOperations(
    statement_handle: HStmt,
    _operation: USmallInt,
) -> SqlReturn {
    unsupported_function!(statement_handle)
}

///
/// [`SQLCancel`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLCancel-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLCancel(statement_handle: HStmt) -> SqlReturn {
    panic_safe_exec_keep_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let stmt = must_be_valid!(mongo_handle.as_statement());

            // use the statement state to determine if a query is executing or not
            match *(stmt.state.read().unwrap()) {
                // if a query is executing, verify we have a connection (we must to be executing a query) and use that connection to kill
                // queries associated with the current statement handle
                StatementState::SynchronousQueryExecuting => {
                    let stmt_id = stmt.statement_id.read().unwrap().clone();
                    let conn = must_be_valid!((*stmt.connection).as_connection());
                    if let Some(mongo_connection) = conn.mongo_connection.read().unwrap().as_ref() {
                        odbc_unwrap!(
                            mongo_connection.cancel_queries_for_statement(stmt_id),
                            try_mongo_handle!(statement_handle)
                        );
                        SqlReturn::SUCCESS
                    } else {
                        stmt.errors
                            .write()
                            .unwrap()
                            .push(ODBCError::InvalidCursorState);
                        SqlReturn::ERROR
                    }
                }
                _ => SqlReturn::SUCCESS,
            }
        },
        statement_handle
    )
}

///
/// [`SQLCancelHandle`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLCancelHandle-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLCancelHandle(_handle_type: HandleType, handle: Handle) -> SqlReturn {
    unimpl!(handle);
}

///
/// [`SQLCloseCursor`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLCloseCursor-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLCloseCursor(_statement_handle: HStmt) -> SqlReturn {
    // We never need to do anything to close a cusor, so this is safe.
    SqlReturn::SUCCESS
}

///
/// [`SQLColAttributeW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLColAttribute-function
///
/// This is the WideChar version of the SQLColAttribute function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLColAttributeW(
    statement_handle: HStmt,
    column_number: USmallInt,
    field_identifier: USmallInt,
    character_attribute_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
    numeric_attribute_ptr: *mut Len,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let odbc_version = mongo_handle.get_odbc_version();
            let stmt = must_be_valid!((*mongo_handle).as_statement());
            let mongo_stmt = stmt.mongo_statement.read().unwrap();
            stmt.errors.write().unwrap().clear();
            if mongo_stmt.is_none() {
                stmt.errors.write().unwrap().push(ODBCError::NoResultSet);
                return SqlReturn::ERROR;
            }
            let max_string_length = stmt.get_max_string_length();
            let string_col_attr = |f: &dyn Fn(&MongoColMetadata) -> &str| {
                let mongo_handle = try_mongo_handle!(statement_handle);
                let col_metadata = mongo_stmt
                    .as_ref()
                    .unwrap()
                    .get_col_metadata(column_number, max_string_length);
                if let Ok(col_metadata) = col_metadata {
                    return i16_len::set_output_wstring_as_bytes(
                        (*f)(col_metadata),
                        character_attribute_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    );
                }
                // unfortunately, we cannot use odbc_unwrap! on the value because it causes a deadlock.
                add_diag_info!(
                    mongo_handle,
                    ODBCError::InvalidDescriptorIndex(column_number)
                );
                SqlReturn::ERROR
            };
            let numeric_col_attr = |f: &dyn Fn(&MongoColMetadata) -> Len| {
                {
                    let col_metadata = mongo_stmt
                        .as_ref()
                        .unwrap()
                        .get_col_metadata(column_number, max_string_length);
                    if let Ok(col_metadata) = col_metadata {
                        *numeric_attribute_ptr = (*f)(col_metadata);
                        return SqlReturn::SUCCESS;
                    }
                }
                // unfortunately, we cannot use odbc_unwrap! on the value because it causes a deadlock.
                stmt.errors
                    .write()
                    .unwrap()
                    .push(ODBCError::InvalidDescriptorIndex(column_number));
                SqlReturn::ERROR
            };
            match FromPrimitive::from_u16(field_identifier) {
                Some(desc) => match desc {
                    Desc::SQL_DESC_AUTO_UNIQUE_VALUE => {
                        *numeric_attribute_ptr = SqlBool::SQL_FALSE as Len;
                        SqlReturn::SUCCESS
                    }
                    Desc::SQL_DESC_UNNAMED | Desc::SQL_DESC_UPDATABLE => {
                        *numeric_attribute_ptr = 0 as Len;
                        SqlReturn::SUCCESS
                    }
                    Desc::SQL_DESC_COUNT => {
                        *numeric_attribute_ptr = isize::try_from(
                            mongo_stmt
                                .as_ref()
                                .unwrap()
                                .get_resultset_metadata(max_string_length)
                                .len(),
                        )
                        .expect("SQL_DESC_COUNT value exceeds isize on this platform");
                        SqlReturn::SUCCESS
                    }
                    Desc::SQL_DESC_CASE_SENSITIVE => {
                        numeric_col_attr(&|x: &MongoColMetadata| isize::from(x.case_sensitive))
                    }
                    Desc::SQL_DESC_BASE_COLUMN_NAME => {
                        string_col_attr(&|x: &MongoColMetadata| x.base_col_name.as_ref())
                    }
                    Desc::SQL_DESC_BASE_TABLE_NAME => {
                        string_col_attr(&|x: &MongoColMetadata| x.base_table_name.as_ref())
                    }
                    Desc::SQL_DESC_CATALOG_NAME => {
                        string_col_attr(&|x: &MongoColMetadata| x.catalog_name.as_ref())
                    }
                    Desc::SQL_DESC_DISPLAY_SIZE => numeric_col_attr(&|x: &MongoColMetadata| {
                        isize::try_from(x.display_size.unwrap_or(0))
                            .expect("display size exceeds isize on this platform")
                    }),
                    Desc::SQL_DESC_FIXED_PREC_SCALE => {
                        numeric_col_attr(&|x: &MongoColMetadata| isize::from(x.fixed_prec_scale))
                    }
                    Desc::SQL_DESC_LABEL => {
                        string_col_attr(&|x: &MongoColMetadata| x.label.as_ref())
                    }
                    Desc::SQL_DESC_LITERAL_PREFIX => {
                        string_col_attr(&|x: &MongoColMetadata| x.literal_prefix.unwrap_or(""))
                    }
                    Desc::SQL_DESC_LITERAL_SUFFIX => {
                        string_col_attr(&|x: &MongoColMetadata| x.literal_suffix.unwrap_or(""))
                    }
                    Desc::SQL_DESC_LOCAL_TYPE_NAME | Desc::SQL_DESC_SCHEMA_NAME => {
                        string_col_attr(&|_| "")
                    }
                    Desc::SQL_DESC_NAME => {
                        string_col_attr(&|x: &MongoColMetadata| x.col_name.as_ref())
                    }
                    Desc::SQL_DESC_NULLABLE => {
                        numeric_col_attr(&|x: &MongoColMetadata| x.nullability as Len)
                    }
                    Desc::SQL_DESC_NUM_PREC_RADIX => numeric_col_attr(&|x: &MongoColMetadata| {
                        isize::try_from(x.num_prec_radix.unwrap_or(0))
                            .expect("num_prec_radix exceeds isize on this platform")
                    }),
                    Desc::SQL_DESC_OCTET_LENGTH | Desc::SQL_COLUMN_LENGTH => {
                        numeric_col_attr(&|x: &MongoColMetadata| {
                            isize::try_from(x.transfer_octet_length.unwrap_or(0))
                                .expect("transfer_octet_length exceeds isize on this platform")
                        })
                    }
                    Desc::SQL_DESC_LENGTH => numeric_col_attr(&|x: &MongoColMetadata| {
                        isize::try_from(x.length.unwrap_or(0))
                            .expect("descriptor length exceeds isize on this platform")
                    }),
                    Desc::SQL_DESC_PRECISION => numeric_col_attr(&|x: &MongoColMetadata| {
                        isize::try_from(x.precision.unwrap_or(0))
                            .expect("descriptor precision exceeds isize on this platform")
                    }),
                    // Column size
                    Desc::SQL_COLUMN_PRECISION => numeric_col_attr(&|x: &MongoColMetadata| {
                        x.column_size
                            .unwrap_or(0)
                            .try_into()
                            .expect("column size exceeds isize on this platform")
                    }),
                    // Decimal digit
                    Desc::SQL_COLUMN_SCALE => numeric_col_attr(&|x: &MongoColMetadata| {
                        x.decimal_digits
                            .unwrap_or(0)
                            .try_into()
                            .expect("decimal digits exceeds isize")
                    }),
                    Desc::SQL_DESC_SCALE => numeric_col_attr(&|x: &MongoColMetadata| {
                        x.scale
                            .unwrap_or(0)
                            .try_into()
                            .expect("scale exceeds isize")
                    }),
                    Desc::SQL_DESC_SEARCHABLE => {
                        numeric_col_attr(&|x: &MongoColMetadata| x.searchable as Len)
                    }
                    Desc::SQL_DESC_TABLE_NAME => {
                        string_col_attr(&|x: &MongoColMetadata| x.table_name.as_ref())
                    }
                    Desc::SQL_DESC_TYPE_NAME => {
                        string_col_attr(&|x: &MongoColMetadata| x.type_name.as_ref())
                    }
                    Desc::SQL_DESC_TYPE | Desc::SQL_DESC_CONCISE_TYPE => {
                        numeric_col_attr(&|x: &MongoColMetadata| {
                            handle_sql_type(odbc_version, x.sql_type) as Len
                        })
                    }
                    Desc::SQL_DESC_UNSIGNED => {
                        numeric_col_attr(&|x: &MongoColMetadata| x.is_unsigned.into())
                    }
                    Desc::SQL_DESC_ALLOC_TYPE => {
                        numeric_col_attr(&|_| AllocType::SQL_DESC_ALLOC_AUTO as Len)
                    }
                    desc @ (Desc::SQL_DESC_OCTET_LENGTH_PTR
                    | Desc::SQL_DESC_DATETIME_INTERVAL_CODE
                    | Desc::SQL_DESC_INDICATOR_PTR
                    | Desc::SQL_DESC_DATA_PTR
                    | Desc::SQL_DESC_ARRAY_SIZE
                    | Desc::SQL_DESC_ARRAY_STATUS_PTR
                    | Desc::SQL_DESC_BIND_OFFSET_PTR
                    | Desc::SQL_DESC_BIND_TYPE
                    | Desc::SQL_DESC_DATETIME_INTERVAL_PRECISION
                    | Desc::SQL_DESC_MAXIMUM_SCALE
                    | Desc::SQL_DESC_MINIMUM_SCALE
                    | Desc::SQL_DESC_PARAMETER_TYPE
                    | Desc::SQL_DESC_ROWS_PROCESSED_PTR
                    | Desc::SQL_DESC_ROWVER) => {
                        let mongo_handle = try_mongo_handle!(statement_handle);
                        let _ = must_be_valid!((*mongo_handle).as_statement());
                        add_diag_info!(
                            mongo_handle,
                            ODBCError::UnsupportedFieldDescriptor(desc as u16)
                        );
                        SqlReturn::ERROR
                    }
                },
                None => {
                    let mongo_handle = try_mongo_handle!(statement_handle);
                    let _ = must_be_valid!((*mongo_handle).as_statement());
                    add_diag_info!(
                        mongo_handle,
                        ODBCError::InvalidFieldDescriptor(field_identifier)
                    );
                    SqlReturn::ERROR
                }
            }
        },
        statement_handle
    );
}

///
/// [`SQLColumnPrivilegesW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLColumnPrivileges-function
///
/// This is the WideChar version of the SQLColumnPrivileges function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLColumnPrivilegesW(
    statement_handle: HStmt,
    _catalog_name: *const WideChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WideChar,
    _schema_name_length: SmallInt,
    _table_name: *const WideChar,
    _table_name_length: SmallInt,
    _column_name: *const WideChar,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unimpl!(statement_handle);
}

///
/// [`SQLColumnsW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLColumns-function
///
/// This is the WideChar version of the SQLColumns function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLColumnsW(
    statement_handle: HStmt,
    catalog_name: *const WideChar,
    catalog_name_length: SmallInt,
    _schema_name: *const WideChar,
    _schema_name_length: SmallInt,
    table_name: *const WideChar,
    table_name_length: SmallInt,
    column_name: *const WideChar,
    column_name_length: SmallInt,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let odbc_3_data_types = has_odbc_3_behavior!(mongo_handle);
            let stmt = must_be_valid!((*mongo_handle).as_statement());
            let catalog_string =
                input_text_to_string_w_allow_null(catalog_name, catalog_name_length.into());
            let catalog = if catalog_name.is_null() || catalog_string.is_empty() {
                None
            } else {
                Some(catalog_string.as_str())
            };
            // ignore schema
            let table_string =
                input_text_to_string_w_allow_null(table_name, table_name_length.into());
            let table = if table_name.is_null() {
                None
            } else {
                Some(table_string.as_str())
            };
            let column_name_string =
                input_text_to_string_w_allow_null(column_name, column_name_length.into());
            let column = if column_name.is_null() {
                None
            } else {
                Some(column_name_string.as_str())
            };
            let connection = must_be_valid!((*stmt.connection).as_connection());
            let type_mode = *connection.type_mode.read().unwrap();
            let max_string_length = *connection.max_string_length.read().unwrap();
            let mongo_statement = Box::new(MongoFields::list_columns(
                connection
                    .mongo_connection
                    .read()
                    .unwrap()
                    .as_ref()
                    .unwrap(),
                Some(
                    stmt.attributes
                        .read()
                        .unwrap()
                        .query_timeout
                        .try_into()
                        .unwrap_or(i32::MAX),
                ),
                catalog,
                table,
                column,
                type_mode,
                max_string_length,
                odbc_3_data_types,
            ));
            *stmt.mongo_statement.write().unwrap() = Some(mongo_statement);
            SqlReturn::SUCCESS
        },
        statement_handle
    );
}

///
/// [`SQLCompleteAsync`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLCompleteAsync-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLCompleteAsync(
    _handle_type: HandleType,
    handle: Handle,
    _async_ret_code_ptr: *mut RetCode,
) -> SqlReturn {
    unsupported_function!(handle)
}

///
/// [`SQLConnectW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLConnect-function
///
/// This is the WideChar version of the SQLConnect function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLConnectW(
    connection_handle: HDbc,
    _server_name: *const WideChar,
    _name_length_1: SmallInt,
    _user_name: *const WideChar,
    _name_length_2: SmallInt,
    _authentication: *const WideChar,
    _name_length_3: SmallInt,
) -> SqlReturn {
    unsupported_function!(connection_handle)
}

///
/// [`SQLCopyDesc`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLCopyDesc-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLCopyDesc(
    _source_desc_handle: HDesc,
    _target_desc_handle: HDesc,
) -> SqlReturn {
    unsupported_function!(_source_desc_handle)
}

///
/// [`SQLDataSourcesW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDataSources-function
///
/// This function is implemented only by the Driver Manager.
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
/**
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLDataSourcesW(
    environment_handle: HEnv,
    _direction: USmallInt,
    _server_name: *mut WideChar,
    _buffer_length_1: SmallInt,
    _name_length_1: *mut SmallInt,
    _description: *mut WideChar,
    _buffer_length_2: SmallInt,
    _name_length_2: *mut SmallInt,
) -> SqlReturn {
    unsupported_function!(environment_handle)
}
*/
///
/// [`SQLDescribeColW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDescribeCol-function
///
/// This is the WideChar version of the SQLDescribeCol function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLDescribeColW(
    hstmt: HStmt,
    col_number: USmallInt,
    col_name: *mut WideChar,
    buffer_length: SmallInt,
    name_length: *mut SmallInt,
    data_type: *mut SqlDataType,
    col_size: *mut ULen,
    decimal_digits: *mut SmallInt,
    nullable: *mut SmallInt,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let stmt_handle = try_mongo_handle!(hstmt);
            let odbc_version = stmt_handle.get_odbc_version();
            {
                let stmt = must_be_valid!(stmt_handle.as_statement());
                let mongo_stmt = stmt.mongo_statement.read().unwrap();
                if mongo_stmt.is_none() {
                    stmt.errors.write().unwrap().push(ODBCError::NoResultSet);
                    return SqlReturn::ERROR;
                }
                let max_string_length = stmt.get_max_string_length();
                let col_metadata = mongo_stmt
                    .as_ref()
                    .unwrap()
                    .get_col_metadata(col_number, max_string_length);
                if let Ok(col_metadata) = col_metadata {
                    *data_type = handle_sql_type(odbc_version, col_metadata.sql_type);
                    *col_size = col_metadata.column_size.unwrap_or(0) as usize;
                    *decimal_digits = col_metadata
                        .decimal_digits
                        .unwrap_or(0)
                        .try_into()
                        .unwrap_or(i16::MAX);
                    *nullable = col_metadata.nullability as i16;
                    return i16_len::set_output_wstring(
                        &col_metadata.label,
                        col_name,
                        buffer_length as usize,
                        name_length,
                    );
                }
            }
            add_diag_info!(stmt_handle, ODBCError::InvalidColumnNumber(col_number));

            SqlReturn::ERROR
        },
        hstmt
    );
}

///
/// [`SQLDescribeParam`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDescribeParam-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLDescribeParam(
    statement_handle: HStmt,
    _parameter_number: USmallInt,
    _data_type_ptr: *mut SqlDataType,
    _parameter_size_ptr: *mut ULen,
    _decimal_digits_ptr: *mut SmallInt,
    _nullable_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function!(statement_handle)
}

///
/// [`SQLDisconnect`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDisconnect-function
///
/// This function is used to disconnect from the data source. It will also free any statements that have not been freed.
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLDisconnect(connection_handle: HDbc) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        info,
        || {
            let conn_handle = try_mongo_handle!(connection_handle);
            let conn = must_be_valid!((*conn_handle).as_connection());

            // Close any open cursors on statements and drop all statements
            if let Ok(mut stmts) = conn.statements.write() {
                stmts.iter().for_each(|stmt| {
                    if let Some(stmt) = (*stmt).as_ref() {
                        if let Some(stmt) = stmt.as_statement() {
                            sql_stmt_close_cursor_helper(stmt);
                        }
                    }
                    // We also deallocate the memory for the statement handles here. We do not share this with SQLFreeHandle
                    // as it would special case the way handles are... handled in that function in a generic way.
                    let _ = Box::from_raw(*stmt);
                });
                stmts.clear();
            }

            // set the mongo_connection to None. This will cause the previous mongo_connection
            // to drop and disconnect.
            if let Some(conn) = conn.mongo_connection.write().unwrap().take() {
                let _ = conn.shutdown();
            }
            SqlReturn::SUCCESS
        },
        connection_handle
    );
}

fn sql_driver_connect(conn: &Connection, odbc_uri_string: &str) -> Result<MongoConnection> {
    let mut odbc_uri = ODBCUri::new(odbc_uri_string.to_string())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let client_options = runtime.block_on(async { odbc_uri.try_into_client_options().await })?;
    odbc_uri
        .remove(&["driver", "dsn"])
        .ok_or(ODBCError::MissingDriverOrDSNProperty)?;

    if let Some(log_level) = odbc_uri.remove(&["loglevel"]) {
        // The connection log level takes precedence over the driver log level
        // Update the logger configuration. This will affect all logging from the application.
        Logger::set_log_level(log_level);
    }

    if let Some(simple) = odbc_uri.remove(&["simple_types_only"]) {
        if simple.eq("0") {
            *conn.type_mode.write().unwrap() = TypeMode::Standard;
        }
    }

    if let Some(enable_max_string_length) = odbc_uri.remove(&["enable_max_string_length"]) {
        if enable_max_string_length.eq("1") {
            *conn.max_string_length.write().unwrap() = Some(constants::DEFAULT_MAX_STRING_LENGTH);
        }
    }

    let mut conn_attrs = conn.attributes.write().unwrap();
    let database = if conn_attrs.current_catalog.is_some() {
        conn_attrs.current_catalog.as_deref().map(|s| s.to_string())
    } else {
        let db = odbc_uri.remove(&["database"]);
        conn_attrs.current_catalog = db.as_ref().cloned();
        db
    };
    let connection_timeout = conn_attrs.connection_timeout;
    let login_timeout = conn_attrs.login_timeout;
    // ODBCError has an impl From mongo_odbc_core::Error, but that does not
    // create an impl From Result<T, mongo_odbc_core::Error> to Result<T, ODBCError>
    // hence this bizarre Ok(func?) pattern.
    Ok(mongo_odbc_core::MongoConnection::connect(
        client_options,
        database,
        connection_timeout,
        login_timeout,
        *conn.type_mode.read().unwrap(),
        Some(runtime),
        *conn.max_string_length.read().unwrap(),
    )?)
}

///
/// [`SQLDriverConnectW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDriverConnect-function
///
/// This is the WideChar version of the SQLDriverConnect function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLDriverConnectW(
    connection_handle: HDbc,
    _window_handle: HWnd,
    in_connection_string: *const WideChar,
    string_length_1: SmallInt,
    out_connection_string: *mut WideChar,
    buffer_length: SmallInt,
    string_length_2: *mut SmallInt,
    driver_completion: USmallInt,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let conn_handle = try_mongo_handle!(connection_handle);
            trace_odbc!(
                info,
                conn_handle,
                format!("Connecting using {DRIVER_NAME} {} ", *DRIVER_ODBC_VERSION),
                function_name!()
            );

            // We will treat any valid option passed for DriverComplete as a no-op
            // because we don't have any UI involved in the process and don't plan to add any in the future
            match <DriverConnectOption as FromPrimitive>::from_u16(driver_completion) {
                Some(_) => {}
                None => {
                    add_diag_info!(
                        conn_handle,
                        ODBCError::InvalidDriverCompletion(driver_completion)
                    );
                    return SqlReturn::ERROR;
                }
            }

            let conn = must_be_valid!((*conn_handle).as_connection());
            let odbc_uri_string =
                input_text_to_string_w(in_connection_string, string_length_1.into());
            let mongo_connection =
                odbc_unwrap!(sql_driver_connect(conn, &odbc_uri_string), conn_handle);
            *conn.mongo_connection.write().unwrap() = Some(mongo_connection);
            // We know the mysql ODBC driver returns SUCCESS if the out_connection_string is NULL.
            // We can also just return SUCCESS if the buffer_len is 0. Likely, users are not
            // expecting to get back a warning when they pass an empty buffer to this, especially
            // given that we only currently support DriverConnectOption::SQL_DRIVER_NO_PROMPT.
            // given that we only currently support DriverConnectOption::NoPrompt.
            if buffer_length <= 0 || out_connection_string.is_null() {
                // Only assign to string_length_2 if it is not null
                if !string_length_2.is_null() {
                    *string_length_2 = odbc_uri_string
                        .len()
                        .try_into()
                        .expect("odbc_uri_string.len exceeds i16");
                }
                return SqlReturn::SUCCESS;
            }
            let buffer_len = usize::try_from(buffer_length).unwrap();
            let sql_return = i16_len::set_output_wstring(
                &odbc_uri_string,
                out_connection_string,
                buffer_len,
                string_length_2,
            );
            if sql_return == SqlReturn::SUCCESS_WITH_INFO {
                add_diag_info!(conn_handle, ODBCError::OutStringTruncated(buffer_len));
            }
            sql_return
        },
        connection_handle
    );
}

///
/// [`SQLDriversW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLDrivers-function
///
/// This function is implemented only by the Driver Manager.
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
/**
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLDriversW(
    henv: HEnv,
    _direction: USmallInt,
    _driver_desc: *mut WideChar,
    _driver_desc_max: SmallInt,
    _out_driver_desc: *mut SmallInt,
    _driver_attributes: *mut WideChar,
    _drvr_attr_max: SmallInt,
    _out_drvr_attr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function!(henv)
}
**/
///
/// [`SQLEndTran`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLEndTran-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLEndTran(
    _handle_type: HandleType,
    handle: Handle,
    _completion_type: SmallInt,
) -> SqlReturn {
    unimpl!(handle);
}

///
/// [`SQLExecDirectW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLExecDirect-function
///
/// This is the WideChar version of the SQLExecDirect function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLExecDirectW(
    statement_handle: HStmt,
    statement_text: *const WideChar,
    text_length: Integer,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let stmt = must_be_valid!(mongo_handle.as_statement());
            let connection = must_be_valid!((*stmt.connection).as_connection());
            let mongo_statement = odbc_unwrap!(
                sql_prepare(statement_text, text_length, connection,),
                mongo_handle
            );

            *stmt.mongo_statement.write().unwrap() = Some(Box::new(mongo_statement));

            // set the statment state to executing so SQLCancel knows to search the op log for hanging queries
            *stmt.state.write().unwrap() = StatementState::SynchronousQueryExecuting;

            odbc_unwrap!(sql_execute(stmt, connection), mongo_handle);

            // return the statement state to its original value
            *stmt.state.write().unwrap() = StatementState::Allocated;

            SqlReturn::SUCCESS
        },
        statement_handle
    );
}

///
/// [`SQLExecute`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLExecute-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLExecute(statement_handle: HStmt) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let stmt = must_be_valid!(mongo_handle.as_statement());
            let connection = must_be_valid!((*stmt.connection).as_connection());
            // set the statment state to executing so SQLCancel knows to search the op log for hanging queries
            *stmt.state.write().unwrap() = StatementState::SynchronousQueryExecuting;
            odbc_unwrap!(sql_execute(stmt, connection), mongo_handle);
            // return the statement state to its original value
            *stmt.state.write().unwrap() = StatementState::Allocated;
            SqlReturn::SUCCESS
        },
        statement_handle
    );
}

unsafe fn sql_execute(stmt: &Statement, connection: &Connection) -> Result<bool> {
    let stmt_id = stmt.statement_id.read().unwrap().clone();
    let mongo_statement = {
        if let Some(mongo_connection) = connection.mongo_connection.read().unwrap().as_ref() {
            let rowset_size = match u32::try_from(stmt.attributes.read().unwrap().row_array_size) {
                Ok(size) => size,
                Err(_) => unreachable!("Err should be impossible since SQLSetStmtAttrW sets row_array_size to u32::MAX if it's outside of the u32 range"),
            };

            stmt.mongo_statement
                .write()
                .unwrap()
                .as_mut()
                .unwrap()
                .execute(mongo_connection, stmt_id, rowset_size)
                .map_err(|e| e.into())
        } else {
            Err(ODBCError::InvalidCursorState)
        }
    };

    mongo_statement
}

///
/// [`SQLFetch`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLFetch-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLFetch(statement_handle: HStmt) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || { sql_fetch_helper(statement_handle, "SQLFetch") },
        statement_handle
    );
}

unsafe fn sql_fetch_helper(statement_handle: HStmt, function_name: &str) -> SqlReturn {
    let mongo_handle = try_mongo_handle!(statement_handle);
    let stmt = must_be_valid!(mongo_handle.as_statement());

    let mut encountered_success_with_info = false;
    let mut global_fetch_warnings_opt: Vec<Error> = Vec::new();

    // needed for rowsets with size > 1 to ensure that NO_DATA does not get returned if the rowset hits the end of the result set.
    let mut has_fetched_at_least_one_row = false;

    let rowset_size = stmt.attributes.read().unwrap().row_array_size;

    // if rows_fetched_ptr is null, it was not set by the user and can be ignored.
    let has_rows_fetched_buffer = !stmt.attributes.read().unwrap().rows_fetched_ptr.is_null();

    // if row_status_ptr is null, it was not set by the user and can be ignored.
    let has_row_status_array = !stmt.attributes.read().unwrap().row_status_ptr.is_null();

    if has_rows_fetched_buffer {
        *stmt.attributes.write().unwrap().rows_fetched_ptr = 0;
    }

    // This variable keeps track of the amount of rows that do not have SQL_ROW_NOROW status.
    // It's necessary because this function needs to know the amount of fetched rows, and
    // the rows_fetched_ptr may not be set by the user, so the function can't depend on the rows_fetched_ptr.
    let mut fetched_rows = 0;

    // keeps track of how many rows in the result set cause SqlReturn::ERROR when being handled.
    let mut row_error_count = 0;

    // Use `index` to figure out which buffer in the array of buffers to use.
    for index in 0..rowset_size {
        // TODO: SQL-2014: Update SQLCancel and SQLFetch so SQLFetch can be cancelled correctly
        // The main idea behind SQL-2014 is to check before each call to next and stop the execution if the statement is cancelled.
        let move_to_next_result = {
            let connection = must_be_valid!((*stmt.connection).as_connection());
            match stmt.mongo_statement.write().unwrap().as_mut() {
                Some(mongo_stmt) => mongo_stmt
                    .next(connection.mongo_connection.read().unwrap().as_ref())
                    .map_err(|e| e.into()),
                None => Err(ODBCError::InvalidCursorState),
            }
        };

        // Set row_status_buffer to the address that corresponds with the current row if the user has set the row_status_ptr.
        // NOTE: The user is responsible for creating a buffer big enough to hold the status for every row in the rowset.
        // If the user does not create a big enough buffer, we will end up overwriting whatever is next in memory,
        // causing undefined behavior.
        let row_status_buffer: *mut USmallInt = if has_row_status_array {
            (stmt.attributes.read().unwrap().row_status_ptr as ULen + (index * size_of::<u16>()))
                as *mut USmallInt
        } else {
            null_mut()
        };

        if let Ok((has_next, mut row_warnings_opt)) = move_to_next_result {
            let mongo_handle = try_mongo_handle!(statement_handle);
            row_warnings_opt.iter().for_each(|warning| {
                add_diag_with_function!(
                    mongo_handle,
                    ODBCError::GeneralWarning(warning.to_string()),
                    function_name.to_string()
                );
            });

            if !has_next {
                // There are no more rows, so set the rest of Row Status Array buffers to SQL_ROW_NOROW if the rowset end has not been reached.
                if has_row_status_array {
                    for row_status_index in index..rowset_size {
                        let row_status_buffer = (stmt.attributes.read().unwrap().row_status_ptr
                            as ULen
                            + (row_status_index * size_of::<u16>()))
                            as *mut USmallInt;
                        *row_status_buffer = RowStatus::SQL_ROW_NOROW as USmallInt;
                    }
                }

                // break here to prevent NO_DATA from being returned if the rowset had at least one row in it.
                if has_fetched_at_least_one_row {
                    break;
                }

                // if the cursor has already reached the end of the result set when SQLFetch is called, NO_DATA is returned.
                // Additionally, rows_fetch_ptr is set to 0.
                return SqlReturn::NO_DATA;
            }

            has_fetched_at_least_one_row = true;

            if has_row_status_array {
                *row_status_buffer = if row_warnings_opt.is_empty() {
                    RowStatus::SQL_ROW_SUCCESS as USmallInt
                } else {
                    RowStatus::SQL_ROW_SUCCESS_WITH_INFO as USmallInt
                };
            }

            // keep track of all warnings that occur throughout the entire function.
            global_fetch_warnings_opt.append(&mut row_warnings_opt);

            fetched_rows += 1;

            *stmt.var_data_cache.write().unwrap() = Some(HashMap::new());

            // If there are bound columns, then copy data from the result set into the bound buffers.
            if let Some(bound_cols) = stmt.bound_cols.read().unwrap().as_ref() {
                let (
                    encountered_error_during_col_binding,
                    encountered_success_with_info_during_col_binding,
                ) = sql_fetch_bound_buffers(
                    statement_handle,
                    index,
                    row_status_buffer,
                    bound_cols,
                    function_name,
                );

                // It's possible that one binding causes SUCCESS_WITH_INFO and another causes ERROR, so we need to check for an error first
                // and ignore SUCCESS_WITH_INFO if it also occurs.
                if encountered_error_during_col_binding {
                    row_error_count += 1;
                }
                // keep track if SUCCESS_WITH_INFO is ever encountered throughout the entire function.
                else if encountered_success_with_info_during_col_binding {
                    encountered_success_with_info = true;
                }
            }
        } else {
            // An error happened when moving the cursor and fetching the next row
            let mongo_handle = try_mongo_handle!(statement_handle);
            add_diag_with_function!(
                mongo_handle,
                move_to_next_result.as_ref().unwrap_err().clone(),
                function_name.to_string()
            );

            let error = move_to_next_result.unwrap_err();

            // Checks if there is an error that applies to the entire function instead of just one row.
            // If there is, early exit with SqlReturn::Error.
            if matches!(error, ODBCError::InvalidCursorState) {
                return SqlReturn::ERROR;
            }

            if has_row_status_array {
                *row_status_buffer = RowStatus::SQL_ROW_ERROR as USmallInt;
            }

            fetched_rows += 1;
            row_error_count += 1;
        }
    }

    if has_rows_fetched_buffer {
        *stmt.attributes.write().unwrap().rows_fetched_ptr = fetched_rows;
    }

    // Only return ERROR if every row that does not have status SQL_ROW_NOROW causes an error.
    if row_error_count == fetched_rows {
        SqlReturn::ERROR
    } else if !global_fetch_warnings_opt.is_empty()
        || encountered_success_with_info
        || row_error_count > 0
    {
        SqlReturn::SUCCESS_WITH_INFO
    } else {
        SqlReturn::SUCCESS
    }
}

/// This is a helper function for SQLFetch that copies data from the current rowset into all the bound buffers.
/// Additionally, it keeps track of any errors that occur during this process and returns two boolean values
/// indicating if SqlReturn::ERROR or SqlReturn::SUCCESS_WITH_INFO were encountered when putting data in the
/// bound buffers. Lastly, if SqlReturn::ERROR or SqlReturn::SUCCESS_WITH_INFO are encountered, this function
/// updates the row status array to reflect that.
unsafe fn sql_fetch_bound_buffers(
    statement_handle: HStmt,
    index: ULen,
    row_status_buffer: *mut USmallInt,
    bound_cols: &HashMap<USmallInt, BoundColInfo>,
    function_name: &str,
) -> (bool, bool) {
    let mongo_handle_for_sql_get_data_helper = match MongoHandleRef::try_from(statement_handle) {
        Ok(h) => h,
        Err(_) => return (true, false), // Returning (true, false) to indicate SqlReturn::ERROR was encountered
    };

    let mut encountered_error_getting_data = false;
    let mut encountered_success_with_info_getting_data = false;

    for (col, bound_col_info) in bound_cols.iter() {
        // Set target_buffer to the correct buffer in the array of buffers
        let target_buffer = (bound_col_info.target_buffer as ULen
            + (index * (bound_col_info.buffer_length as ULen)))
            as Pointer;

        // Set length/indicator buffer to the correct buffer in the array of buffers
        let len_ind_buffer =
            (bound_col_info.length_or_indicator as ULen + (index * size_of::<isize>())) as *mut Len;

        let sql_return = sql_get_data_helper(
            mongo_handle_for_sql_get_data_helper,
            *col,
            FromPrimitive::from_i16(bound_col_info.target_type).unwrap(), // this conversion is checked in SQLBindCol, so it is guaranteed to work here.
            target_buffer,
            bound_col_info.buffer_length,
            len_ind_buffer,
            function_name,
        );

        match sql_return {
            SqlReturn::ERROR => {
                encountered_error_getting_data = true;
            }
            SqlReturn::SUCCESS_WITH_INFO => {
                encountered_success_with_info_getting_data = true;
            }
            _ => {}
        }
    }

    if !row_status_buffer.is_null() {
        // It's possible that one binding causes SUCCESS_WITH_INFO and another causes ERROR, so we need to check for an error first
        // and ignore SUCCESS_WITH_INFO if it also occurs.
        if encountered_error_getting_data {
            *row_status_buffer = RowStatus::SQL_ROW_ERROR as USmallInt;
        } else if encountered_success_with_info_getting_data {
            *row_status_buffer = RowStatus::SQL_ROW_SUCCESS_WITH_INFO as USmallInt;
        }
    }

    (
        encountered_error_getting_data,
        encountered_success_with_info_getting_data,
    )
}

///
/// [`SQLFetchScroll`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLFetchScroll-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLFetchScroll(
    statement_handle: HStmt,
    fetch_orientation: SmallInt,
    _fetch_offset: Len,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            match FromPrimitive::from_i32(i32::from(fetch_orientation)) {
                Some(FetchOrientation::SQL_FETCH_NEXT) => {
                    sql_fetch_helper(statement_handle, "SQLFetchScroll")
                }
                _ => {
                    let stmt_handle = try_mongo_handle!(statement_handle);
                    add_diag_info!(
                        stmt_handle,
                        ODBCError::FetchTypeOutOfRange(fetch_orientation)
                    );
                    SqlReturn::ERROR
                }
            }
        },
        statement_handle
    );
}

///
/// [`SQLForeignKeysW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLForeignKeys-function
///
/// This is the WideChar version of the SQLForeignKeys function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLForeignKeysW(
    statement_handle: HStmt,
    _pk_catalog_name: *const WideChar,
    _pk_catalog_name_length: SmallInt,
    _pk_schema_name: *const WideChar,
    _pk_schema_name_length: SmallInt,
    _pk_table_name: *const WideChar,
    _pk_table_name_length: SmallInt,
    _fk_catalog_name: *const WideChar,
    _fk_catalog_name_length: SmallInt,
    _fk_schema_name: *const WideChar,
    _fk_schema_name_length: SmallInt,
    _fk_table_name: *const WideChar,
    _fk_table_name_length: SmallInt,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let stmt = must_be_valid!((*mongo_handle).as_statement());
            let max_string_length = stmt.get_max_string_length();
            let mongo_statement = MongoForeignKeys::empty(max_string_length);
            *stmt.mongo_statement.write().unwrap() = Some(Box::new(mongo_statement));
            SqlReturn::SUCCESS
        },
        statement_handle
    );
}

///
/// [`SQLFreeHandle`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLFreeHandle-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLFreeHandle(handle_type: HandleType, handle: Handle) -> SqlReturn {
    trace_odbc!(
        info,
        *handle.cast::<MongoHandle>(),
        format!("Freeing handle {:?}", handle.cast::<MongoHandle>()),
        function_name!()
    );
    panic_safe_exec_keep_diagnostics!(
        debug,
        || {
            match sql_free_handle(handle_type, handle.cast()) {
                Ok(_) => SqlReturn::SUCCESS,
                Err(_) => SqlReturn::INVALID_HANDLE,
            }
        },
        null_mut() as Handle
    );
}

fn sql_free_handle(handle_type: HandleType, handle: *mut MongoHandle) -> Result<()> {
    match handle_type {
        // By making Boxes to the types and letting them go out of
        // scope, they will be dropped.
        HandleType::SQL_HANDLE_ENV => {
            let _ = unsafe {
                (*handle)
                    .as_env()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_ENV_ERROR))?
            };
        }
        HandleType::SQL_HANDLE_DBC => {
            let conn = unsafe {
                (*handle)
                    .as_connection()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_CONN_ERROR))?
            };
            let env = unsafe {
                (*conn.env)
                    .as_env()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_ENV_ERROR))?
            };
            let mut connections = env.connections.write().unwrap();
            connections.remove(&handle);
            if connections.is_empty() {
                *env.state.write().unwrap() = EnvState::Allocated;
            }
        }
        HandleType::SQL_HANDLE_STMT => {
            let stmt = unsafe {
                (*handle)
                    .as_statement()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_STMT_ERROR))?
            };
            // Ensure the cursor is closed on the statement before dropping it.
            sql_stmt_close_cursor_helper(stmt);
            // Actually reading this value would make ASAN fail, but this
            // is what the ODBC standard expects.
            let conn = unsafe {
                (*stmt.connection)
                    .as_connection()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_CONN_ERROR))?
            };
            let mut statements = conn.statements.write().unwrap();
            statements.remove(&handle);
            if statements.is_empty() {
                *conn.state.write().unwrap() = ConnectionState::Connected;
            }
        }
        HandleType::SQL_HANDLE_DESC => {
            let _ = unsafe {
                (*handle)
                    .as_descriptor()
                    .ok_or(ODBCError::InvalidHandleType(HANDLE_MUST_BE_DESC_ERROR))?
            };
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
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLFreeStmt(statement_handle: HStmt, option: SmallInt) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let stmt = must_be_valid!((*mongo_handle).as_statement());

            match FromPrimitive::from_i16(option) {
                // Drop all pending results from the cursor and close the cursor.
                Some(FreeStmtOption::SQL_CLOSE) => {
                    let mut mongo_statement = stmt.mongo_statement.write().unwrap();
                    match mongo_statement.as_mut() {
                        // No-op when the mongo_statement is not set. This is typically an
                        // invalid workflow, but we have observed some tools attempt this.
                        None => SqlReturn::SUCCESS,
                        // When the mongo_statement is set, close the cursor.
                        Some(mongo_statement) => {
                            mongo_statement.close_cursor();
                            SqlReturn::SUCCESS
                        }
                    }
                }
                // Release all column buffers bound by SQLBindCol by removing the bound_cols map.
                Some(FreeStmtOption::SQL_UNBIND) => {
                    *stmt.bound_cols.write().unwrap() = None;
                    SqlReturn::SUCCESS
                }
                // We do not implement SQLBindParameter, so this is a no-op.
                Some(FreeStmtOption::SQL_RESET_PARAMS) => SqlReturn::SUCCESS,
                _ => SqlReturn::ERROR,
            }
        },
        statement_handle
    )
}

///
/// [`SQLGetConnectAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetConnectAttr-function
///
/// This is the WideChar version of the SQLGetConnectAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLGetConnectAttrW(
    connection_handle: HDbc,
    attribute: Integer,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let conn_handle = try_mongo_handle!(connection_handle);

            match FromPrimitive::from_i32(attribute) {
                Some(valid_attr) => sql_get_connect_attrw_helper(
                    conn_handle,
                    valid_attr,
                    value_ptr,
                    buffer_length,
                    string_length_ptr,
                ),
                None => {
                    add_diag_info!(conn_handle, ODBCError::InvalidAttrIdentifier(attribute));
                    SqlReturn::ERROR
                }
            }
        },
        connection_handle
    )
}

unsafe fn sql_get_connect_attrw_helper(
    conn_handle: &mut MongoHandle,
    attribute: ConnectionAttribute,
    value_ptr: Pointer,
    buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    let mut err = None;

    // This scope is introduced to make the RWLock Guard expire before we write
    // any error values via add_diag_info as RWLock::write is not reentrant on
    // all operating systems, and the docs say it can panic.
    let sql_return = {
        let conn = must_be_valid!((*conn_handle).as_connection());
        let attributes = &conn.attributes.read().unwrap();

        match attribute {
            ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG => {
                let current_catalog = attributes.current_catalog.as_deref();
                match current_catalog {
                    None => SqlReturn::NO_DATA,
                    Some(cc) => i32_len::set_output_wstring_as_bytes(
                        cc,
                        value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    ),
                }
            }
            ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT => {
                let login_timeout = attributes.login_timeout.unwrap_or(0);
                i32_len::set_output_fixed_data(&login_timeout, value_ptr, string_length_ptr)
            }
            // according to the spec, SQL_ATTR_CONNECTION_DEAD just returns the latest status of the connection, not the current status
            ConnectionAttribute::SQL_ATTR_CONNECTION_DEAD => {
                let connection_dead = match *conn.mongo_connection.read().unwrap() {
                    Some(_) => SqlBool::SQL_FALSE,
                    None => SqlBool::SQL_TRUE,
                };
                i32_len::set_output_fixed_data(&connection_dead, value_ptr, string_length_ptr)
            }
            ConnectionAttribute::SQL_ATTR_CONNECTION_TIMEOUT => {
                let connection_timeout = attributes.connection_timeout.unwrap_or(0);
                i32_len::set_output_fixed_data(&connection_timeout, value_ptr, string_length_ptr)
            }
            ConnectionAttribute::SQL_ATTR_ACCESS_MODE => {
                i32_len::set_output_fixed_data(&AccessMode::ReadOnly, value_ptr, string_length_ptr)
            }
            _ => {
                err = Some(ODBCError::UnsupportedConnectionAttribute(
                    connection_attribute_to_string(attribute),
                ));
                SqlReturn::ERROR
            }
        }
    };

    if let Some(e) = err {
        add_diag_with_function!(conn_handle, e, "SQLGetConnectAttrW");
    }
    sql_return
}

///
/// [`SQLGetCursorNameW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetCursorName-function
///
/// This is the WideChar version of the SQLGetCursorName function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLGetCursorNameW(
    statement_handle: HStmt,
    _cursor_name: *mut WideChar,
    _buffer_length: SmallInt,
    _name_length_ptr: *mut SmallInt,
) -> SqlReturn {
    unimpl!(statement_handle);
}

///
/// [`SQLGetData`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetData-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLGetData(
    statement_handle: HStmt,
    col_or_param_num: USmallInt,
    target_type: SmallInt,
    target_value_ptr: Pointer,
    buffer_length: Len,
    str_len_or_ind_ptr: *mut Len,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let stmt = must_be_valid!((*mongo_handle).as_statement());

            // Make sure that SQLGetData only runs when dealing with rowsets of size 1.
            if stmt.attributes.read().unwrap().row_array_size != 1 {
                let mongo_handle = try_mongo_handle!(statement_handle);
                add_diag_info!(
                    mongo_handle,
                    ODBCError::Unimplemented("`SQLGetData with rowset size greater than 1`")
                );
                return SqlReturn::ERROR;
            }

            match FromPrimitive::from_i16(target_type) {
                Some(valid_type) => sql_get_data_helper(
                    mongo_handle,
                    col_or_param_num,
                    valid_type,
                    target_value_ptr,
                    buffer_length,
                    str_len_or_ind_ptr,
                    "SQLGetData",
                ),
                None => {
                    add_diag_info!(mongo_handle, ODBCError::InvalidTargetType(target_type));
                    SqlReturn::ERROR
                }
            }
        },
        statement_handle
    )
}

unsafe fn sql_get_data_helper(
    mongo_handle: &mut MongoHandle,
    col_or_param_num: USmallInt,
    target_type: CDataType,
    target_value_ptr: Pointer,
    buffer_length: Len,
    str_len_or_ind_ptr: *mut Len,
    function_name: &str,
) -> SqlReturn {
    let mut error = None;
    let mut ret = Bson::Null;
    {
        let res = {
            let stmt = must_be_valid!((*mongo_handle).as_statement());
            stmt.var_data_cache
                .write()
                .unwrap()
                .as_mut()
                .unwrap()
                .remove(&col_or_param_num)
        };
        if let Some(cached_data) = res {
            return crate::api::data::format_cached_data(
                mongo_handle,
                cached_data,
                col_or_param_num,
                target_type,
                target_value_ptr,
                buffer_length,
                str_len_or_ind_ptr,
                function_name,
            );
        }
        let stmt = (*mongo_handle).as_statement().unwrap();
        let mut mongo_stmt = stmt.mongo_statement.write().unwrap();
        let max_string_length = stmt.get_max_string_length();
        let bson = match mongo_stmt.as_mut() {
            None => Err(ODBCError::InvalidCursorState),
            Some(mongo_stmt) => mongo_stmt
                .get_value(col_or_param_num, max_string_length)
                .map_err(ODBCError::Core),
        };
        match bson {
            Err(e) => error = Some(e),
            Ok(None) => {
                ret = Bson::Null;
            }
            Ok(Some(d)) => {
                ret = d;
            }
        }
    }
    if let Some(e) = error {
        add_diag_with_function!(mongo_handle, e, function_name);
        return SqlReturn::ERROR;
    }
    crate::api::data::format_bson_data(
        mongo_handle,
        col_or_param_num,
        target_type,
        target_value_ptr,
        buffer_length,
        str_len_or_ind_ptr,
        ret,
        function_name,
    )
}

///
/// [`SQLGetDescFieldW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDescField-function
///
/// This is the WideChar version of the SQLGetDescField function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLGetDescFieldW(
    _descriptor_handle: HDesc,
    _record_number: SmallInt,
    _field_identifier: SmallInt,
    _value_ptr: Pointer,
    _buffer_length: Integer,
    _string_length_ptr: *mut Integer,
) -> SqlReturn {
    unsupported_function!(_descriptor_handle)
}

///
/// [`SQLGetDescRecW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDescRec-function
///
/// This is the WideChar version of the SQLGetDescRec function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLGetDescRecW(
    _descriptor_handle: HDesc,
    _record_number: SmallInt,
    _name: *mut WideChar,
    _buffer_length: SmallInt,
    _string_length_ptr: *mut SmallInt,
    _type_ptr: *mut SmallInt,
    _sub_type_ptr: *mut SmallInt,
    _length_ptr: *mut Len,
    _precision_ptr: *mut SmallInt,
    _scale_ptr: *mut SmallInt,
    _nullable_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function!(_descriptor_handle)
}

///
/// [`SQLGetDiagFieldW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDiagField-function
///
/// This is the WideChar version of the SQLGetDiagField function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLGetDiagFieldW(
    _handle_type: HandleType,
    handle: Handle,
    record_number: SmallInt,
    diag_identifier: SmallInt,
    diag_info_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    panic_safe_exec_keep_diagnostics!(
        debug,
        || {
            let mongo_handle = handle.cast::<MongoHandle>();
            let odbc_version = (*mongo_handle).get_odbc_version();
            let get_error = |errors: &Vec<ODBCError>, diag_identifier: DiagType| -> SqlReturn {
                get_diag_fieldw(
                    errors,
                    diag_identifier,
                    odbc_version,
                    diag_info_ptr,
                    record_number,
                    buffer_length,
                    string_length_ptr,
                )
            };

            match FromPrimitive::from_i16(diag_identifier) {
                Some(diag_identifier) => {
                    match diag_identifier {
                        // some diagnostics are statement specific; return error if another handle is passed
                        DiagType::SQL_DIAG_ROW_COUNT | DiagType::SQL_DIAG_ROW_NUMBER => {
                            if _handle_type != HandleType::SQL_HANDLE_STMT {
                                return SqlReturn::ERROR;
                            }
                            get_stmt_diag_field(diag_identifier, diag_info_ptr)
                        }
                        DiagType::SQL_DIAG_NUMBER
                        | DiagType::SQL_DIAG_MESSAGE_TEXT
                        | DiagType::SQL_DIAG_NATIVE
                        | DiagType::SQL_DIAG_SQLSTATE
                        | DiagType::SQL_DIAG_RETURNCODE => match _handle_type {
                            HandleType::SQL_HANDLE_ENV => {
                                let env = must_be_env!(mongo_handle);
                                get_error(&env.errors.read().unwrap(), diag_identifier)
                            }
                            HandleType::SQL_HANDLE_DBC => {
                                let dbc = must_be_conn!(mongo_handle);
                                get_error(&dbc.errors.read().unwrap(), diag_identifier)
                            }
                            HandleType::SQL_HANDLE_STMT => {
                                let stmt = must_be_stmt!(mongo_handle);
                                get_error(&stmt.errors.read().unwrap(), diag_identifier)
                            }
                            HandleType::SQL_HANDLE_DESC => {
                                let desc = must_be_desc!(mongo_handle);
                                get_error(&desc.errors.read().unwrap(), diag_identifier)
                            }
                        },
                        // TODO: SQL-1152: Implement additional diag types
                        // this condition should only occur if the _diag_identifier is not in the spec
                        _ => SqlReturn::ERROR,
                    }
                }
                None => SqlReturn::ERROR,
            }
        },
        handle
    )
}

macro_rules! sql_get_diag_rec_impl {
    ($handle_type:ident, $handle:ident, $rec_number:ident, $state:ident, $native_error_ptr:ident, $message_text:ident, $buffer_length:ident, $text_length_ptr:ident, $error_output_func:ident) => {{
        panic_safe_exec_keep_diagnostics!(
            debug,
            || {
                if $rec_number < 1 || $buffer_length < 0 {
                    return SqlReturn::ERROR;
                }
                let mongo_handle = $handle.cast::<MongoHandle>();
                let odbc_version = (*mongo_handle).get_odbc_version();
                // Make the record number zero-indexed
                let rec_number = ($rec_number - 1) as usize;

                let get_error = |errors: &Vec<ODBCError>| -> SqlReturn {
                    match errors.get(rec_number) {
                        Some(odbc_err) => $error_output_func(
                            odbc_err,
                            $state,
                            odbc_version,
                            $message_text,
                            $buffer_length,
                            $text_length_ptr,
                            $native_error_ptr,
                        ),
                        None => SqlReturn::NO_DATA,
                    }
                };

                match $handle_type {
                    HandleType::SQL_HANDLE_ENV => {
                        let env = must_be_env!(mongo_handle);
                        get_error(&env.errors.read().unwrap())
                    }
                    HandleType::SQL_HANDLE_DBC => {
                        let dbc = must_be_conn!(mongo_handle);
                        get_error(&dbc.errors.read().unwrap())
                    }
                    HandleType::SQL_HANDLE_STMT => {
                        let stmt = must_be_stmt!(mongo_handle);
                        get_error(&stmt.errors.read().unwrap())
                    }
                    HandleType::SQL_HANDLE_DESC => {
                        let desc = must_be_desc!(mongo_handle);
                        get_error(&desc.errors.read().unwrap())
                    }
                }
            },
            $handle
        );
    }};
}

///
/// [`SQLGetDiagRecW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetDiagRec-function
///
/// This is the WideChar version of the SQLGetDiagRec function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLGetDiagRecW(
    handle_type: HandleType,
    handle: Handle,
    rec_number: SmallInt,
    state: *mut WideChar,
    native_error_ptr: *mut Integer,
    message_text: *mut WideChar,
    buffer_length: SmallInt,
    text_length_ptr: *mut SmallInt,
) -> SqlReturn {
    sql_get_diag_rec_impl!(
        handle_type,
        handle,
        rec_number,
        state,
        native_error_ptr,
        message_text,
        buffer_length,
        text_length_ptr,
        get_diag_recw
    )
}

///
/// [`SQLGetEnvAttr`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetEnvAttr-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLGetEnvAttr(
    environment_handle: HEnv,
    attribute: Integer,
    value_ptr: Pointer,
    _buffer_length: Integer,
    string_length: *mut Integer,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        info,
        || {
            let env_handle = try_mongo_handle!(environment_handle);

            match FromPrimitive::from_i32(attribute) {
                Some(valid_attr) => {
                    sql_get_env_attrw_helper(env_handle, valid_attr, value_ptr, string_length)
                }
                None => {
                    add_diag_info!(env_handle, ODBCError::InvalidAttrIdentifier(attribute));
                    SqlReturn::ERROR
                }
            }
        },
        environment_handle
    );
}

unsafe fn sql_get_env_attrw_helper(
    env_handle: &MongoHandle,
    attribute: EnvironmentAttribute,
    value_ptr: Pointer,
    string_length: *mut Integer,
) -> SqlReturn {
    let env = must_be_valid!(env_handle.as_env());
    if value_ptr.is_null() {
        ptr_safe_write(string_length, 0);
    } else {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        ptr_safe_write(string_length, size_of::<Integer>() as Integer);
        match attribute {
            EnvironmentAttribute::SQL_ATTR_ODBC_VERSION => {
                *value_ptr.cast::<AttrOdbcVersion>() = env.attributes.read().unwrap().odbc_ver;
            }
            EnvironmentAttribute::SQL_ATTR_OUTPUT_NTS => {
                *value_ptr.cast::<SqlBool>() = env.attributes.read().unwrap().output_nts;
            }
            EnvironmentAttribute::SQL_ATTR_CONNECTION_POOLING => {
                *value_ptr.cast::<AttrConnectionPooling>() =
                    env.attributes.read().unwrap().connection_pooling;
            }
            EnvironmentAttribute::SQL_ATTR_CP_MATCH => {
                *value_ptr.cast::<AttrCpMatch>() = env.attributes.read().unwrap().cp_match;
            }
            EnvironmentAttribute::SQL_ATTR_DRIVER_UNICODE_TYPE => {
                *value_ptr.cast::<Charset>() = env.attributes.read().unwrap().driver_unicode_type;
            }
        }
    }
    SqlReturn::SUCCESS
}

macro_rules! sql_get_info_helper {
($connection_handle:ident,
 $info_type:ident,
 $info_value_ptr:ident,
 $buffer_length:ident,
 $string_length_ptr:ident,
 ) => {{
    use constants::*;
    use definitions::InfoType;
    let connection_handle = $connection_handle;
    let info_type = $info_type;
    let info_value_ptr = $info_value_ptr;
    let buffer_length = $buffer_length;
    let string_length_ptr = $string_length_ptr;

    let conn_handle = try_mongo_handle!(connection_handle);
    let mut err = None;
    let sql_return = match FromPrimitive::from_u16(info_type) {
        Some(some_info_type) => {
            trace_odbc!(
                debug,
                conn_handle,
                format!("InfoType {some_info_type:?}"),
                "SQLGetInfoW"
            );
            match some_info_type {
                InfoType::SQL_DRIVER_NAME => {
                    // This Driver Name is consistent with the name used for our JDBC driver.
                    i16_len::set_output_wstring_as_bytes(
                        DRIVER_NAME,
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_DRIVER_VER => i16_len::set_output_wstring_as_bytes(
                    DRIVER_ODBC_VERSION.as_str(),
                    info_value_ptr,
                    buffer_length as usize,
                    string_length_ptr,
                ),
                InfoType::SQL_DRIVER_ODBC_VER => {
                    // This driver supports version 3.8.
                    i16_len::set_output_wstring(
                        ODBC_VERSION,
                        info_value_ptr.cast::<WideChar>(),
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_SEARCH_PATTERN_ESCAPE => i16_len::set_output_wstring_as_bytes(
                    r"\",
                    info_value_ptr,
                    buffer_length as usize,
                    string_length_ptr,
                ),
                InfoType::SQL_DBMS_NAME => {
                    // The underlying DBMS is MongoDB Atlas.
                    i16_len::set_output_wstring_as_bytes(
                        DBMS_NAME,
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_DBMS_VER => {
                    // Return the ADF version.
                    let conn = must_be_valid!((*conn_handle).as_connection());
                    let version = conn
                        .mongo_connection
                        .read()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .get_adf_version();
                    match version {
                        Ok(version) => i16_len::set_output_wstring_as_bytes(
                            version.as_str(),
                            info_value_ptr,
                            buffer_length as usize,
                            string_length_ptr,
                        ),
                        Err(e) => {
                            err = Some(ODBCError::Core(e));
                            SqlReturn::ERROR
                        }
                    }
                }
                InfoType::SQL_CONCAT_NULL_BEHAVIOR => {
                    // If a NULL valued operand is used in a string concatenation,
                    // the result is NULL. The return value indicates that.
                    i16_len::set_output_fixed_data(&SQL_CB_NULL, info_value_ptr, string_length_ptr)
                }
                InfoType::SQL_IDENTIFIER_QUOTE_CHAR => {
                    // MongoSQL supports ` and " as identifier delimiters. The "
                    // character is the SQL-92 standard, but we instead return `
                    // to be consistent with our JDBC driver.
                    i16_len::set_output_wstring_as_bytes(
                        "`",
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_OWNER_TERM => {
                    // SQL_OWNER_TERM is replaced by SQL_SCHEMA_TERM in newer ODBC
                    // versions. They use the same numeric value.
                    //
                    // SQL has two concepts in the data hierarchy above "table":
                    // "catalog" and "schema". MongoSQL only has "database" and
                    // "collection" (which is equivalent to "table"). A "catalog"
                    // contains many "schemas" and a "schema" contains many tables.
                    // Therefore, a "schema" may map to MongoSQL's "database".
                    // However, we choose to use "catalog" to represent MongoSQL
                    // databases, and we omit support for "schema".
                    i16_len::set_output_wstring_as_bytes(
                        "",
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_OJ_CAPABILITIES => {
                    const OJ_CAPABILITIES: u32 = SQL_OJ_LEFT
                      | SQL_OJ_NOT_ORDERED
                      | SQL_OJ_INNER
                      | SQL_OJ_ALL_COMPARISON_OPS;
                    i16_len::set_output_fixed_data(
                        &OJ_CAPABILITIES,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_CATALOG_NAME_SEPARATOR => {
                    // The name separator used by MongoSQL is '.'.
                    i16_len::set_output_wstring_as_bytes(
                        ".",
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_CATALOG_TERM => {
                    // MongoSQL uses the term "database".
                    i16_len::set_output_wstring_as_bytes(
                        "database",
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_CONVERT_FUNCTIONS => {
                    // MongoSQL only supports the CAST type conversion function.
                    i16_len::set_output_fixed_data(
                        &SQL_FN_CVT_CAST,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_NUMERIC_FUNCTIONS => {
                    // MongoSQL supports the following numeric functions.
                    const NUMERIC_FUNCTIONS: u32 = SQL_FN_NUM_ABS
                        | SQL_FN_NUM_CEILING
                        | SQL_FN_NUM_COS
                        | SQL_FN_NUM_FLOOR
                        | SQL_FN_NUM_LOG
                        | SQL_FN_NUM_MOD
                        | SQL_FN_NUM_SIN
                        | SQL_FN_NUM_SQRT
                        | SQL_FN_NUM_TAN
                        | SQL_FN_NUM_DEGREES
                        | SQL_FN_NUM_POWER
                        | SQL_FN_NUM_RADIANS
                        | SQL_FN_NUM_LOG
                        | SQL_FN_NUM_LOG10
                        | SQL_FN_NUM_ROUND;
                    i16_len::set_output_fixed_data(
                        &NUMERIC_FUNCTIONS,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_STRING_FUNCTIONS => {
                    // MongoSQL supports the following string functions.
                    const STRING_FUNCTIONS: u32 = SQL_FN_STR_CONCAT
                        | SQL_FN_STR_LENGTH
                        | SQL_FN_STR_SUBSTRING
                        | SQL_FN_STR_BIT_LENGTH
                        | SQL_FN_STR_CHAR_LENGTH
                        | SQL_FN_STR_CHARACTER_LENGTH
                        | SQL_FN_STR_OCTET_LENGTH
                        | SQL_FN_STR_POSITION
                        | SQL_FN_STR_UCASE
                        | SQL_FN_STR_LCASE
                        | SQL_FN_STR_REPLACE;

                    i16_len::set_output_fixed_data(
                        &STRING_FUNCTIONS,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_SYSTEM_FUNCTIONS => {
                    // MongoSQL does not support any of the ODBC system functions.
                    i16_len::set_output_fixed_data(&MAX_COLUMNS_U32_ZERO, info_value_ptr, string_length_ptr)
                }
                InfoType::SQL_TIMEDATE_FUNCTIONS => {
                    // MongoSQL supports the following timedate functions.
                    const TIMEDATE_FUNCTIONS: u32 = SQL_FN_TD_CURRENT_TIMESTAMP
                        | SQL_FN_TD_NOW
                        | SQL_FN_TD_TIMESTAMPADD
                        | SQL_FN_TD_TIMESTAMPDIFF
                        | SQL_FN_TD_EXTRACT
                        | SQL_FN_TD_YEAR
                        | SQL_FN_TD_MONTH
                        | SQL_FN_TD_WEEK
                        | SQL_FN_TD_DAYOFWEEK
                        | SQL_FN_TD_DAYOFYEAR
                        | SQL_FN_TD_DAYOFMONTH
                        | SQL_FN_TD_HOUR
                        | SQL_FN_TD_MINUTE
                        | SQL_FN_TD_SECOND;
                    i16_len::set_output_fixed_data(
                        &TIMEDATE_FUNCTIONS,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_CONVERT_BIGINT
                | InfoType::SQL_CONVERT_DECIMAL
                | InfoType::SQL_CONVERT_DOUBLE
                | InfoType::SQL_CONVERT_FLOAT
                | InfoType::SQL_CONVERT_INTEGER
                | InfoType::SQL_CONVERT_NUMERIC
                | InfoType::SQL_CONVERT_REAL
                | InfoType::SQL_CONVERT_SMALLINT
                | InfoType::SQL_CONVERT_TINYINT
                | InfoType::SQL_CONVERT_BIT
                | InfoType::SQL_CONVERT_CHAR
                | InfoType::SQL_CONVERT_VARCHAR
                | InfoType::SQL_CONVERT_LONGVARCHAR
                | InfoType::SQL_CONVERT_WCHAR
                | InfoType::SQL_CONVERT_WVARCHAR
                | InfoType::SQL_CONVERT_WLONGVARCHAR
                | InfoType::SQL_CONVERT_TIMESTAMP
                | InfoType::SQL_CONVERT_BINARY
                | InfoType::SQL_CONVERT_DATE
                | InfoType::SQL_CONVERT_TIME
                | InfoType::SQL_CONVERT_VARBINARY
                | InfoType::SQL_CONVERT_LONGVARBINARY
                | InfoType::SQL_CONVERT_GUID => {
                    // MongoSQL does not support the CONVERT scalar function, but clients also use
                    // this to apply the CAST syntactic construct. The value we return for
                    // SQL_CONVERT_FUNCTIONS alerts the client that we expect CAST and not CONVERT.
                    i16_len::set_output_fixed_data(&MONGO_CAST_SUPPORT, info_value_ptr, string_length_ptr)
                }
                InfoType::SQL_GETDATA_EXTENSIONS => {
                    // GetData can be called on any column in any order.
                    const GETDATA_EXTENSIONS: u32 = SQL_GD_ANY_COLUMN | SQL_GD_ANY_ORDER;
                    i16_len::set_output_fixed_data(
                        &GETDATA_EXTENSIONS,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_COLUMN_ALIAS => {
                    // MongoSQL does support column aliases.
                    i16_len::set_output_wstring_as_bytes(
                        COLUMN_ALIAS_INFO_Y,
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_GROUP_BY => {
                    // The GROUP BY clause must contain all nonaggregated columns
                    // in the select list. It can contain columns that are not in
                    // the select list.
                    i16_len::set_output_fixed_data(
                        &SQL_GB_GROUP_BY_CONTAINS_SELECT,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_ORDER_BY_COLUMNS_IN_SELECT => {
                    // MongoSQL does require ORDER BY columns to be in the SELECT list.
                    i16_len::set_output_wstring_as_bytes(
                        COLUMN_ALIAS_INFO_Y,
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_OWNER_USAGE => {
                    // SQL_OWNER_USAGE is replaced by SQL_SCHEMA_USAGE in newer
                    // ODBC versions. They use the same numeric value.
                    //
                    // As noted for InfoType::OwnerTerm, the MongoSQL ODBC driver
                    // does not support "schema" in the data hierarchy.
                    i16_len::set_output_fixed_data(&MAX_COLUMNS_U32_ZERO, info_value_ptr, string_length_ptr)
                }
                InfoType::SQL_CATALOG_USAGE => {
                    // This return value indicates support for SELECT as well as
                    // INSERT, UPDATE, and DELETE. In conjunction with the following
                    // InfoType, SQL_DATA_SOURCE_READ_ONLY, this return value is
                    // valid.
                    i16_len::set_output_fixed_data(
                        &SQL_CU_DML_STATEMENTS,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_DATA_SOURCE_READ_ONLY => {
                    // MongoSQL is read-only.
                    i16_len::set_output_wstring_as_bytes(
                        COLUMN_ALIAS_INFO_Y,
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_SPECIAL_CHARACTERS => {
                    // According to the ODBC spec, this InfoType requires returning "A
                    // character string that contains all special characters (that is,
                    // all characters except a through z, A through Z, 0 through 9, and
                    // underscore) that can be used in an identifier name... For example,
                    // '#$^'. If an identifier contains one or more of these characters,
                    // the identifier must be a delimited identifier."
                    //
                    // MongoSQL grammar defines regular and delimited identifiers as
                    //
                    //    <regular identifier> ::= ([A-Za-z] | "_")[A-Za-z0-9_]*
                    //
                    //    <delimited identifier> ::= " <identifier character>* "
                    //                             | ` <identifier character>* `
                    //
                    //    <identifier character> ::= [^\x00]
                    //
                    // Meaning, MongoSQL requires delimiters for all characters other
                    // than [A-Za-z0-9_]. It is unrealistic to return a string with
                    // all of those characters, so here we choose to return a string
                    // containing what we believe to be most common special characters.
                    i16_len::set_output_wstring_as_bytes(
                        "`\"'.$+-*/|:<>!={}[]()",
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_MAX_COLUMNS_IN_GROUP_BY
                | InfoType::SQL_MAX_COLUMNS_IN_ORDER_BY
                | InfoType::SQL_MAX_COLUMNS_IN_SELECT => {
                    // MongoSQL does not have an explicit maximum number of
                    // columns allowed in a GROUP BY, ORDER BY, or SELECT clause.
                    i16_len::set_output_fixed_data(&MAX_COLUMNS_U16_ZERO, info_value_ptr, string_length_ptr)
                }

                InfoType::SQL_TIMEDATE_ADD_INTERVALS | InfoType::SQL_TIMEDATE_DIFF_INTERVALS => {
                    // Note that MongoSQL does not support TIMEDATE_ADD or
                    // TIMEDATE_DIFF, so this value will not be used. For the
                    // MongoSQL DATEADD and DATEDIFF functions, we support the
                    // following intervals.
                    const TIMEDATE_INTERVALS: u32 = SQL_FN_TSI_FRAC_SECOND
                        | SQL_FN_TSI_SECOND
                        | SQL_FN_TSI_MINUTE
                        | SQL_FN_TSI_HOUR
                        | SQL_FN_TSI_DAY
                        | SQL_FN_TSI_WEEK
                        | SQL_FN_TSI_MONTH
                        | SQL_FN_TSI_QUARTER
                        | SQL_FN_TSI_YEAR;
                    i16_len::set_output_fixed_data(
                        &TIMEDATE_INTERVALS,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_CATALOG_LOCATION => {
                    // MongoSQL puts the catalog (database) at the start of a qualified
                    // table name. As in, db.table.
                    i16_len::set_output_fixed_data(&SQL_CL_START, info_value_ptr, string_length_ptr)
                }
                InfoType::SQL_SQL_CONFORMANCE => {
                    // MongoSQL is SQL-92 Entry level compliant.
                    i16_len::set_output_fixed_data(
                        &SQL_SC_SQL92_ENTRY,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_ODBC_INTERFACE_CONFORMANCE => {
                    // The MongoSQL ODBC Driver currently meets the minimum compliance level.
                    i16_len::set_output_fixed_data(&SQL_OIC_CORE, info_value_ptr, string_length_ptr)
                }
                InfoType::SQL_SQL92_PREDICATES => {
                    // MongoSQL supports the following SQL-92 predicate operators.
                    const PREDICATES: u32 = SQL_SP_EXISTS
                        | SQL_SP_ISNOTNULL
                        | SQL_SP_ISNULL
                        | SQL_SP_LIKE
                        | SQL_SP_IN
                        | SQL_SP_BETWEEN
                        | SQL_SP_COMPARISON
                        | SQL_SP_QUANTIFIED_COMPARISON;
                    i16_len::set_output_fixed_data(&PREDICATES, info_value_ptr, string_length_ptr)
                }
                InfoType::SQL_SQL92_RELATIONAL_JOIN_OPERATORS => {
                    // MongoSQL supports the following SQL-92 JOIN operators.
                    const JOIN_OPS: u32 = SQL_SRJO_CROSS_JOIN
                        | SQL_SRJO_INNER_JOIN
                        | SQL_SRJO_LEFT_OUTER_JOIN;
                    i16_len::set_output_fixed_data(&JOIN_OPS, info_value_ptr, string_length_ptr)
                }
                InfoType::SQL_AGGREGATE_FUNCTIONS => {
                    // MongoSQL supports the following aggregate functions.
                    const AGG_FUNCTIONS: u32 = SQL_AF_AVG
                        | SQL_AF_COUNT
                        | SQL_AF_MAX
                        | SQL_AF_MIN
                        | SQL_AF_SUM
                        | SQL_AF_DISTINCT
                        | SQL_AF_ALL;
                    i16_len::set_output_fixed_data(
                        &AGG_FUNCTIONS,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_CATALOG_NAME => {
                    // MongoSQL does support catalog (database) names.
                    i16_len::set_output_wstring_as_bytes(
                        COLUMN_ALIAS_INFO_Y,
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_MAX_IDENTIFIER_LEN => {
                    // MongoSQL does not have a maximum identifier length.
                    // Note : The spec does not specify which value to return if there are no maximum
                    // Let's report the max value for SQLUSMALLINT.
                    i16_len::set_output_fixed_data(&u16::MAX, info_value_ptr, string_length_ptr)
                }
                // Since we don't support transaction, Commit and Rollback are not supported.
                InfoType::SQL_CURSOR_COMMIT_BEHAVIOR | InfoType::SQL_CURSOR_ROLLBACK_BEHAVIOR => {
                    i16_len::set_output_fixed_data(
                        &SQL_CB_PRESERVE,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_DEFAULT_TXN_ISOLATION
                | InfoType::SQL_DTC_TRANSITION_COST
                | InfoType::SQL_BOOKMARK_PERSISTENCE
                | InfoType::SQL_POS_OPERATIONS
                | InfoType::SQL_STATIC_SENSITIVITY
                | InfoType::SQL_TXN_CAPABLE => {
                    i16_len::set_output_fixed_data(
                        &0,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                // Setting this to 10, which is our default for the number of workers in the mongo driver's connection pool.
                InfoType::SQL_MAX_CONCURRENT_ACTIVITIES => {
                    i16_len::set_output_fixed_data(&10, info_value_ptr, string_length_ptr)
                }
                InfoType::SQL_FORWARD_ONLY_CURSOR_ATTRIBUTES1
                | InfoType::SQL_KEYSET_CURSOR_ATTRIBUTES1
                | InfoType::SQL_DYNAMIC_CURSOR_ATTRIBUTES1
                | InfoType::SQL_STATIC_CURSOR_ATTRIBUTES1 => {
                    i16_len::set_output_fixed_data(
                        &SQL_CA1_NEXT,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_FORWARD_ONLY_CURSOR_ATTRIBUTES2
                | InfoType::SQL_KEYSET_CURSOR_ATTRIBUTES2
                | InfoType::SQL_DYNAMIC_CURSOR_ATTRIBUTES2
                | InfoType::SQL_STATIC_CURSOR_ATTRIBUTES2 => {
                    i16_len::set_output_fixed_data(
                        &MONGO_CA2_SUPPORT,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_SCROLL_OPTIONS => {
                    i16_len::set_output_fixed_data(
                        &MONGO_SO_SUPPORT,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_NEED_LONG_DATA_LEN => {
                    i16_len::set_output_wstring_as_bytes(
                        COLUMN_ALIAS_INFO_Y,
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_TXN_ISOLATION_OPTION => {
                    i16_len::set_output_fixed_data(
                        &SQL_TXN_SERIALIZABLE,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_DATABASE_NAME => {
                    let conn = must_be_valid!((*conn_handle).as_connection());
                    let attributes = conn.attributes.read().unwrap();
                    if attributes.current_catalog.is_some() {
                        i16_len::set_output_wstring_as_bytes(
                            attributes.current_catalog.as_ref().unwrap().as_str(),
                            info_value_ptr,
                            buffer_length as usize,
                            string_length_ptr,
                        )
                    } else {
                        err = Some(ODBCError::ConnectionNotOpen);
                        SqlReturn::ERROR
                    }
                }
                InfoType::SQL_SCROLL_CONCURRENCY => {
                    i16_len::set_output_fixed_data(
                        &SQL_SCCO_READ_ONLY,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_LOCK_TYPES => {
                    i16_len::set_output_fixed_data(
                        &SQL_LCK_NO_CHANGE,
                        info_value_ptr,
                        string_length_ptr,
                    )
                }
                InfoType::SQL_KEYWORDS => {
                    i16_len::set_output_wstring_as_bytes(
                        KEYWORDS.as_str(),
                        info_value_ptr,
                        buffer_length as usize,
                        string_length_ptr,
                    )
                }
                _ => {
                    err = Some(ODBCError::UnsupportedInfoTypeRetrieval(
                        info_type.to_string(),
                    ));
                    SqlReturn::ERROR
                }
            }
        }
        None => {
            err = Some(ODBCError::UnsupportedInfoTypeRetrieval(
                info_type.to_string(),
            ));
            SqlReturn::ERROR
        }
    };

    if let Some(error) = err {
        add_diag_with_function!(conn_handle, error, "SQLGetInfoW");
    }
    sql_return
}}
}

///
/// [`SQLGetInfoW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetInfo-function
///
/// This is the WideChar version of the SQLGetInfo function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLGetInfoW(
    connection_handle: HDbc,
    info_type: USmallInt,
    info_value_ptr: Pointer,
    buffer_length: SmallInt,
    string_length_ptr: *mut SmallInt,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || sql_get_info_helper!(
            connection_handle,
            info_type,
            info_value_ptr,
            buffer_length,
            string_length_ptr,
        ),
        connection_handle
    )
}

///
/// [`SQLGetStmtAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetStmtAttr-function
///
/// This is the WideChar version of the SQLGetStmtAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLGetStmtAttrW(
    handle: HStmt,
    attribute: Integer,
    value_ptr: Pointer,
    _buffer_length: Integer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let stmt_handle = try_mongo_handle!(handle);
            if value_ptr.is_null() {
                return SqlReturn::ERROR;
            }

            match FromPrimitive::from_i32(attribute) {
                Some(valid_attr) => {
                    sql_get_stmt_attrw_helper(stmt_handle, valid_attr, value_ptr, string_length_ptr)
                }
                None => {
                    add_diag_info!(stmt_handle, ODBCError::InvalidAttrIdentifier(attribute));
                    SqlReturn::ERROR
                }
            }
        },
        handle
    );
}

// Allowing as these lints are from size_of coercion
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
unsafe fn sql_get_stmt_attrw_helper(
    stmt_handle: &mut MongoHandle,
    attribute: StatementAttribute,
    value_ptr: Pointer,
    string_length_ptr: *mut Integer,
) -> SqlReturn {
    // Most attributes have type SQLULEN, so default to the size of that
    // type.
    ptr_safe_write(string_length_ptr, size_of::<ULen>() as Integer);

    let mut err = None;

    let sql_return = {
        let stmt = must_be_valid!(stmt_handle.as_statement());
        match attribute {
            StatementAttribute::SQL_ATTR_APP_ROW_DESC => {
                *value_ptr.cast::<Pointer>() =
                    stmt.attributes.read().unwrap().app_row_desc as Pointer;
                ptr_safe_write(string_length_ptr, size_of::<Pointer>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_APP_PARAM_DESC => {
                *value_ptr.cast::<Pointer>() =
                    stmt.attributes.read().unwrap().app_param_desc as Pointer;
                ptr_safe_write(string_length_ptr, size_of::<Pointer>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_IMP_ROW_DESC => {
                *value_ptr.cast::<Pointer>() =
                    stmt.attributes.read().unwrap().imp_row_desc as Pointer;
                ptr_safe_write(string_length_ptr, size_of::<Pointer>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_IMP_PARAM_DESC => {
                *value_ptr.cast::<Pointer>() =
                    stmt.attributes.read().unwrap().imp_param_desc as Pointer;
                ptr_safe_write(string_length_ptr, size_of::<Pointer>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_FETCH_BOOKMARK_PTR => {
                *value_ptr.cast() = stmt.attributes.read().unwrap().fetch_bookmark_ptr;
                ptr_safe_write(string_length_ptr, size_of::<*mut Len>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_CURSOR_SCROLLABLE => {
                *value_ptr.cast::<CursorScrollable>() =
                    stmt.attributes.read().unwrap().cursor_scrollable;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_CURSOR_SENSITIVITY => {
                *value_ptr.cast::<CursorSensitivity>() =
                    stmt.attributes.read().unwrap().cursor_sensitivity;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_ASYNC_ENABLE => {
                *value_ptr.cast::<AsyncEnable>() = stmt.attributes.read().unwrap().async_enable;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_CONCURRENCY => {
                *value_ptr.cast::<Concurrency>() = stmt.attributes.read().unwrap().concurrency;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_CURSOR_TYPE => {
                *value_ptr.cast::<CursorType>() = stmt.attributes.read().unwrap().cursor_type;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_ENABLE_AUTO_IPD => {
                *value_ptr.cast::<SqlBool>() = stmt.attributes.read().unwrap().enable_auto_ipd;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_KEYSET_SIZE => {
                *value_ptr.cast::<ULen>() = 0;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_MAX_LENGTH => {
                *value_ptr.cast::<ULen>() = stmt.attributes.read().unwrap().max_length;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_MAX_ROWS => {
                *value_ptr.cast::<ULen>() = stmt.attributes.read().unwrap().max_rows;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_NOSCAN => {
                *value_ptr.cast::<NoScan>() = stmt.attributes.read().unwrap().no_scan;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_PARAM_BIND_OFFSET_PTR => {
                *value_ptr.cast() = stmt.attributes.read().unwrap().param_bind_offset_ptr;
                ptr_safe_write(string_length_ptr, size_of::<*mut ULen>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_PARAM_BIND_TYPE => {
                *value_ptr.cast::<ULen>() = stmt.attributes.read().unwrap().param_bind_type;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_PARAM_OPERATION_PTR => {
                *value_ptr.cast() = stmt.attributes.read().unwrap().param_operation_ptr;
                ptr_safe_write(string_length_ptr, size_of::<*mut USmallInt>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_PARAM_STATUS_PTR => {
                *value_ptr.cast() = stmt.attributes.read().unwrap().param_status_ptr;
                ptr_safe_write(string_length_ptr, size_of::<*mut USmallInt>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_PARAMS_PROCESSED_PTR => {
                *value_ptr.cast() = stmt.attributes.read().unwrap().param_processed_ptr;
                ptr_safe_write(string_length_ptr, size_of::<*mut ULen>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_PARAMSET_SIZE => {
                *value_ptr.cast::<ULen>() = stmt.attributes.read().unwrap().paramset_size;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_QUERY_TIMEOUT => {
                *value_ptr.cast::<ULen>() = stmt.attributes.read().unwrap().query_timeout;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_RETRIEVE_DATA => {
                *value_ptr.cast::<RetrieveData>() = stmt.attributes.read().unwrap().retrieve_data;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_ROW_BIND_OFFSET_PTR => {
                *value_ptr.cast() = stmt.attributes.read().unwrap().row_bind_offset_ptr;
                ptr_safe_write(string_length_ptr, size_of::<*mut ULen>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_ROW_BIND_TYPE => {
                *value_ptr.cast::<ULen>() = stmt.attributes.read().unwrap().row_bind_type;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_ROW_NUMBER => {
                *value_ptr.cast::<ULen>() = stmt.attributes.read().unwrap().row_number;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_ROW_OPERATION_PTR => {
                *value_ptr.cast() = stmt.attributes.read().unwrap().row_operation_ptr;
                ptr_safe_write(string_length_ptr, size_of::<*mut USmallInt>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_ROW_STATUS_PTR => {
                *value_ptr.cast() = stmt.attributes.read().unwrap().row_status_ptr;
                ptr_safe_write(string_length_ptr, size_of::<*mut USmallInt>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_ROWS_FETCHED_PTR => {
                *value_ptr.cast() = stmt.attributes.read().unwrap().rows_fetched_ptr;
                ptr_safe_write(string_length_ptr, size_of::<*mut ULen>() as Integer);
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_ROW_ARRAY_SIZE | StatementAttribute::SQL_ROWSET_SIZE => {
                *value_ptr.cast::<ULen>() = stmt.attributes.read().unwrap().row_array_size;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_SIMULATE_CURSOR => {
                *value_ptr.cast::<ULen>() = stmt.attributes.read().unwrap().simulate_cursor;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_USE_BOOKMARKS => {
                *value_ptr.cast::<UseBookmarks>() = stmt.attributes.read().unwrap().use_bookmarks;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_ASYNC_STMT_EVENT => {
                *value_ptr.cast() = stmt.attributes.read().unwrap().async_stmt_event;
                SqlReturn::SUCCESS
            }
            StatementAttribute::SQL_ATTR_METADATA_ID => {
                // False means that we treat arguments to catalog functions as case sensitive. This
                // is a _requirement_ for mongodb where FOO and foo are distinct database names.
                *value_ptr.cast::<ULen>() = SqlBool::SQL_FALSE as ULen;
                SqlReturn::SUCCESS
            }
            // leave SQL_GET_BOOKMARK as unsupported since it is for ODBC < 3.0 drivers
            StatementAttribute::SQL_GET_BOOKMARK
            // Not supported but still relevent to 3.0 drivers
            | StatementAttribute::SQL_ATTR_SAMPLE_SIZE
            | StatementAttribute::SQL_ATTR_DYNAMIC_COLUMNS
            | StatementAttribute::SQL_ATTR_TYPE_EXCEPTION_BEHAVIOR
            | StatementAttribute::SQL_ATTR_LENGTH_EXCEPTION_BEHAVIOR => {
                err = Some(ODBCError::UnsupportedStatementAttribute(
                    statement_attribute_to_string(attribute),
                ));
                SqlReturn::ERROR
            }
        }
    };

    if let Some(error) = err {
        add_diag_with_function!(stmt_handle, error, "SQLGetStmtAttrW");
    }
    sql_return
}

///
/// [`SQLGetTypeInfoW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLGetTypeInfo-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLGetTypeInfoW(handle: HStmt, data_type: SmallInt) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(handle);
            let odbc_version = mongo_handle.get_odbc_version();
            match FromPrimitive::from_i16(data_type) {
                Some(sql_data_type) => {
                    let sql_data_type = handle_sql_type(odbc_version, sql_data_type);
                    let stmt = must_be_valid!((*mongo_handle).as_statement());
                    let type_mode = if stmt.connection.is_null() {
                        TypeMode::Standard
                    } else {
                        let connection = must_be_valid!((*stmt.connection).as_connection());
                        *connection.type_mode.read().unwrap()
                    };
                    let types_info = MongoTypesInfo::new(sql_data_type, type_mode);
                    *stmt.mongo_statement.write().unwrap() = Some(Box::new(types_info));
                    SqlReturn::SUCCESS
                }
                None => {
                    add_diag_info!(
                        mongo_handle,
                        ODBCError::InvalidSqlType(data_type.to_string())
                    );
                    SqlReturn::ERROR
                }
            }
        },
        handle
    )
}

///
/// [`SQLMoreResults`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLMoreResults-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
pub unsafe extern "C" fn SQLMoreResults(_handle: HStmt) -> SqlReturn {
    // For now, we never allow more than one result from a query (i.e., we only support one query
    // at a time).
    SqlReturn::NO_DATA
}

///
/// [`SQLNativeSqlW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLNativeSql-function
///
/// This is the WideChar version of the SQLNativeSql function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLNativeSqlW(
    connection_handle: HDbc,
    _in_statement_text: *const WideChar,
    _in_statement_len: Integer,
    _out_statement_text: *mut WideChar,
    _buffer_len: Integer,
    _out_statement_len: *mut Integer,
) -> SqlReturn {
    unimpl!(connection_handle);
}

///
/// [`SQLNumParams`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLNumParams-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLNumParams(
    statement_handle: HStmt,
    _param_count_ptr: *mut SmallInt,
) -> SqlReturn {
    unsupported_function!(statement_handle)
}

///
/// [`SQLNumResultCols`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLNumResultCols-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLNumResultCols(
    statement_handle: HStmt,
    column_count_ptr: *mut SmallInt,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);

            let stmt = must_be_valid!((*mongo_handle).as_statement());
            let max_string_length = stmt.get_max_string_length();

            let mongo_statement = stmt.mongo_statement.read().unwrap();
            if mongo_statement.is_none() {
                *column_count_ptr = 0;
                return SqlReturn::SUCCESS;
            }
            *column_count_ptr = mongo_statement
                .as_ref()
                .unwrap()
                .get_resultset_metadata(max_string_length)
                .len()
                .try_into()
                .expect("column count exceeded {i16::MAX}");

            SqlReturn::SUCCESS
        },
        statement_handle
    );
}

///
/// [`SQLParamData`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLParamData-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLParamData(hstmt: HStmt, _value_ptr_ptr: *mut Pointer) -> SqlReturn {
    unsupported_function!(hstmt)
}

///
/// [`SQLPrepareW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLPrepare-function
///
/// This is the WideChar version of the SQLPrepare function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLPrepareW(
    statement_handle: HStmt,
    statement_text: *const WideChar,
    text_length: Integer,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let stmt = must_be_valid!(mongo_handle.as_statement());
            let connection = must_be_valid!((*stmt.connection).as_connection());
            let mongo_statement = odbc_unwrap!(
                sql_prepare(statement_text, text_length, connection,),
                mongo_handle
            );

            *stmt.mongo_statement.write().unwrap() = Some(Box::new(mongo_statement));
            SqlReturn::SUCCESS
        },
        statement_handle
    );
}

fn sql_prepare(
    statement_text: *const WideChar,
    text_length: Integer,
    connection: &Connection,
) -> Result<MongoQuery> {
    let mut query = unsafe {
        input_text_to_string_w(
            statement_text,
            text_length
                .try_into()
                .expect("i32 exceeded max isize on this platform"),
        )
    };
    query = query.strip_suffix(';').unwrap_or(&query).to_string();
    let mongo_statement = {
        let type_mode = *connection.type_mode.read().unwrap();
        let max_string_length = *connection.max_string_length.read().unwrap();
        let attributes = connection.attributes.read().unwrap();
        let timeout = attributes.connection_timeout;
        let current_db = attributes.current_catalog.as_ref().cloned();
        if let Some(mongo_connection) = connection.mongo_connection.read().unwrap().as_ref() {
            MongoQuery::prepare(
                mongo_connection,
                current_db,
                timeout,
                &query,
                type_mode,
                max_string_length,
            )
            .map_err(|e| e.into())
        } else {
            Err(ODBCError::InvalidCursorState)
        }
    };
    mongo_statement
}

///
/// [`SQLPrimaryKeysW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLPrimaryKeys-function
///
/// This is the WideChar version of the SQLPrimaryKeys function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLPrimaryKeysW(
    statement_handle: HStmt,
    _catalog_name: *const WideChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WideChar,
    _schema_name_length: SmallInt,
    _table_name: *const WideChar,
    _table_name_length: SmallInt,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let stmt = must_be_valid!((*mongo_handle).as_statement());
            let max_string_length = stmt.get_max_string_length();
            let mongo_statement = MongoPrimaryKeys::empty(max_string_length);
            *stmt.mongo_statement.write().unwrap() = Some(Box::new(mongo_statement));
            SqlReturn::SUCCESS
        },
        statement_handle
    );
}

///
/// [`SQLProcedureColumnsW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLProcedureColumns-function
///
/// This is the WideChar version of the SQLProcedureColumns function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLProcedureColumnsW(
    statement_handle: HStmt,
    _catalog_name: *const WideChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WideChar,
    _schema_name_length: SmallInt,
    _proc_name: *const WideChar,
    _proc_name_length: SmallInt,
    _column_name: *const WideChar,
    _column_name_length: SmallInt,
) -> SqlReturn {
    unsupported_function!(statement_handle)
}

///
/// [`SQLProceduresW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLProcedures-function
///
/// This is the WideChar version of the SQLProcedures function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLProceduresW(
    statement_handle: HStmt,
    _catalog_name: *const WideChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WideChar,
    _schema_name_length: SmallInt,
    _proc_name: *const WideChar,
    _proc_name_length: SmallInt,
) -> SqlReturn {
    unsupported_function!(statement_handle)
}

///
/// [`SQLPutData`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLPutData-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLPutData(
    statement_handle: HStmt,
    _data_ptr: Pointer,
    _str_len_or_ind_ptr: Len,
) -> SqlReturn {
    unsupported_function!(statement_handle)
}

///
/// [`SQLRowCount`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLRowCount-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLRowCount(
    statement_handle: HStmt,
    row_count_ptr: *mut Len,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            // even though we always return 0, we must still assert that the proper handle
            // type is sent by the client.
            let _ = must_be_valid!((*mongo_handle).as_statement());
            *row_count_ptr = 0 as Len;
            SqlReturn::SUCCESS
        },
        statement_handle
    );
}

///
/// [`SQLSetConnectAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetConnectAttr-function
///
/// This is the WideChar version of the SQLSetConnectAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLSetConnectAttrW(
    connection_handle: HDbc,
    attribute: Integer,
    value_ptr: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let conn_handle = try_mongo_handle!(connection_handle);

            match FromPrimitive::from_i32(attribute) {
                Some(valid_attr) => set_connect_attrw_helper(conn_handle, valid_attr, value_ptr),
                None => {
                    add_diag_info!(conn_handle, ODBCError::InvalidAttrIdentifier(attribute));
                    SqlReturn::ERROR
                }
            }
        },
        connection_handle
    )
}

unsafe fn set_connect_attrw_helper(
    conn_handle: &mut MongoHandle,
    attribute: ConnectionAttribute,
    value_ptr: Pointer,
) -> SqlReturn {
    let mut err = None;

    // This scope is introduced to make the RWLock Guard expire before we write
    // any error values via add_diag_info as RWLock::write is not reentrant on
    // all operating systems, and the docs say it can panic.
    let sql_return = {
        let conn = must_be_valid!((*conn_handle).as_connection());

        match attribute {
            ConnectionAttribute::SQL_ATTR_LOGIN_TIMEOUT => {
                conn.attributes.write().unwrap().login_timeout = Some(value_ptr as u32);
                SqlReturn::SUCCESS
            }
            ConnectionAttribute::SQL_ATTR_APP_WCHAR_TYPE => SqlReturn::SUCCESS,
            ConnectionAttribute::SQL_ATTR_CURRENT_CATALOG => {
                let current_db = input_text_to_string_w(
                    value_ptr as *const _,
                    SQL_NTS
                        .try_into()
                        .expect("i32 exceeded max isize on this platform"),
                );
                conn.attributes.write().unwrap().current_catalog = Some(current_db);
                SqlReturn::SUCCESS
            }
            // we use 0 (no timeout throughout the driver); only allow the user to set this value if they are setting to 0
            ConnectionAttribute::SQL_ATTR_CONNECTION_TIMEOUT => match (value_ptr as u32) == 0 {
                true => SqlReturn::SUCCESS,
                false => {
                    conn_handle.add_diag_info(ODBCError::GeneralWarning(
                        "Driver only accepts 0 for SQL_ATTR_CONNECTION_TIMEOUT".into(),
                    ));
                    SqlReturn::SUCCESS_WITH_INFO
                }
            },
            ConnectionAttribute::SQL_ATTR_ACCESS_MODE => {
                match FromPrimitive::from_u32(value_ptr as u32) {
                    Some(AccessMode::ReadOnly) => SqlReturn::SUCCESS,
                    Some(AccessMode::ReadWrite) => {
                        conn_handle.add_diag_info(ODBCError::OptionValueChanged(
                            "SQL_MODE_READ_WRITE",
                            "SQL_MODE_READ",
                        ));
                        SqlReturn::SUCCESS_WITH_INFO
                    }
                    None => {
                        conn_handle
                            .add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_ACCESS_MODE"));
                        SqlReturn::ERROR
                    }
                }
            }
            _ => {
                err = Some(ODBCError::UnsupportedConnectionAttribute(
                    connection_attribute_to_string(attribute),
                ));
                SqlReturn::ERROR
            }
        }
    };

    if let Some(error) = err {
        add_diag_with_function!(conn_handle, error, "SQLSetConnectAttrW");
    }
    sql_return
}

///
/// [`SQLSetCursorNameW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetCursorName-function
///
/// This is the WideChar version of the SQLSetCursorName function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLSetCursorNameW(
    statement_handle: HStmt,
    _cursor_name: *const WideChar,
    _name_length: SmallInt,
) -> SqlReturn {
    unimpl!(statement_handle);
}

///
/// [`SQLSetDescFieldW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetDescField-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLSetDescFieldW(
    _desc_handle: HDesc,
    _rec_number: SmallInt,
    _field_identifier: SmallInt,
    _value_ptr: Pointer,
    _buffer_length: Integer,
) -> SqlReturn {
    unsupported_function!(_desc_handle)
}

///
/// [`SQLSetDescRec`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetDescRec-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLSetDescRec(
    desc_handle: HDesc,
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
    unimpl!(desc_handle)
}

///
/// [`SQLSetPos`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetPos-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLSetPos(
    statement_handle: HStmt,
    _row_number: ULen,
    _operation: USmallInt,
    _lock_type: USmallInt,
) -> SqlReturn {
    unsupported_function!(statement_handle)
}

///
/// [`SQLSetEnvAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetEnvAttr-function
///
/// This is the WideChar version of the SQLSetEnvAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLSetEnvAttr(
    environment_handle: HEnv,
    attribute: Integer,
    value: Pointer,
    _string_length: Integer,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        info,
        || {
            let env_handle = try_mongo_handle!(environment_handle);

            match FromPrimitive::from_i32(attribute) {
                Some(valid_attr) => sql_set_env_attrw_helper(env_handle, valid_attr, value),
                None => {
                    add_diag_info!(env_handle, ODBCError::InvalidAttrIdentifier(attribute));
                    SqlReturn::ERROR
                }
            }
        },
        environment_handle
    );
}

unsafe fn sql_set_env_attrw_helper(
    env_handle: &mut MongoHandle,
    attribute: EnvironmentAttribute,
    value_ptr: Pointer,
) -> SqlReturn {
    let env = must_be_valid!(env_handle.as_env());
    match attribute {
        EnvironmentAttribute::SQL_ATTR_ODBC_VERSION => {
            match FromPrimitive::from_i32(value_ptr as i32) {
                Some(version) => {
                    env.attributes.write().unwrap().odbc_ver = version;
                    SqlReturn::SUCCESS
                }
                None => {
                    add_diag_with_function!(
                        env_handle,
                        ODBCError::InvalidAttrValue("SQL_ATTR_ODBC_VERSION"),
                        "SQLSetEnvAttrW"
                    );
                    SqlReturn::ERROR
                }
            }
        }
        EnvironmentAttribute::SQL_ATTR_OUTPUT_NTS => {
            match FromPrimitive::from_i32(value_ptr as i32) {
                Some(SqlBool::SQL_TRUE) => SqlReturn::SUCCESS,
                _ => {
                    add_diag_with_function!(
                        env_handle,
                        ODBCError::Unimplemented("OUTPUT_NTS=SQL_FALSE"),
                        "SQLSetEnvAttrW"
                    );
                    SqlReturn::ERROR
                }
            }
        }
        EnvironmentAttribute::SQL_ATTR_CONNECTION_POOLING => {
            match FromPrimitive::from_i32(value_ptr as i32) {
                Some(AttrConnectionPooling::SQL_CP_OFF) => SqlReturn::SUCCESS,
                _ => {
                    env_handle.add_diag_info(ODBCError::OptionValueChanged(
                        "SQL_ATTR_CONNECTION_POOLING",
                        "SQL_CP_OFF",
                    ));
                    SqlReturn::SUCCESS_WITH_INFO
                }
            }
        }
        EnvironmentAttribute::SQL_ATTR_CP_MATCH => {
            match FromPrimitive::from_i32(value_ptr as i32) {
                Some(AttrCpMatch::SQL_CP_STRICT_MATCH) => SqlReturn::SUCCESS,
                _ => {
                    env_handle.add_diag_info(ODBCError::OptionValueChanged(
                        "SQL_ATTR_CP_MATCH",
                        "SQL_CP_STRICT_MATCH",
                    ));
                    SqlReturn::SUCCESS_WITH_INFO
                }
            }
        }
        EnvironmentAttribute::SQL_ATTR_DRIVER_UNICODE_TYPE => {
            match FromPrimitive::from_i32(value_ptr as i32) {
                Some(Charset::Utf16) => SqlReturn::SUCCESS,
                Some(Charset::Utf32) => SqlReturn::SUCCESS,
                _ => {
                    env_handle.add_diag_info(ODBCError::OptionValueChanged(
                        "SQL_ATTR_DRIVER_UNICODE_TYPE",
                        "SQL_DM_CP_UTF16",
                    ));
                    SqlReturn::SUCCESS_WITH_INFO
                }
            }
        }
    }
}

///
/// [`SQLSetStmtAttrW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSetStmtAttr-function
///
/// This is the WideChar version of the SQLSetStmtAttr function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLSetStmtAttrW(
    hstmt: HStmt,
    attr: Integer,
    value: Pointer,
    _str_length: Integer,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let stmt_handle = try_mongo_handle!(hstmt);

            match FromPrimitive::from_i32(attr) {
                Some(valid_attr) => sql_set_stmt_attrw_helper(stmt_handle, valid_attr, value),
                None => {
                    add_diag_info!(stmt_handle, ODBCError::InvalidAttrIdentifier(attr));
                    SqlReturn::ERROR
                }
            }
        },
        hstmt
    );
}

unsafe fn sql_set_stmt_attrw_helper(
    stmt_handle: &mut MongoHandle,
    attribute: StatementAttribute,
    value_ptr: Pointer,
) -> SqlReturn {
    let stmt = must_be_valid!(stmt_handle.as_statement());
    match attribute {
        StatementAttribute::SQL_ATTR_APP_ROW_DESC => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_APP_ROW_DESC"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_APP_PARAM_DESC => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_APP_PARAM_DESC"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_IMP_ROW_DESC => {
            // TODO: SQL_681, determine the correct SQL state
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_IMP_ROW_DESC"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_IMP_PARAM_DESC => {
            // TODO: SQL_681, determine the correct SQL state
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_IMP_PARAM_DESC"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_CURSOR_SCROLLABLE => {
            match FromPrimitive::from_usize(value_ptr as usize) {
                Some(CursorScrollable::SQL_NONSCROLLABLE) => SqlReturn::SUCCESS,
                _ => {
                    stmt_handle
                        .add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_CURSOR_SCROLLABLE"));
                    SqlReturn::ERROR
                }
            }
        }
        StatementAttribute::SQL_ATTR_CURSOR_SENSITIVITY => {
            match FromPrimitive::from_i32(value_ptr as i32) {
                Some(CursorSensitivity::SQL_INSENSITIVE) => SqlReturn::SUCCESS,
                _ => {
                    stmt_handle
                        .add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_CURSOR_SENSITIVITY"));
                    SqlReturn::ERROR
                }
            }
        }
        StatementAttribute::SQL_ATTR_ASYNC_ENABLE => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_ASYNC_ENABLE"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_CONCURRENCY => match FromPrimitive::from_i32(value_ptr as i32)
        {
            Some(Concurrency::SQL_CONCUR_READ_ONLY) => SqlReturn::SUCCESS,
            _ => {
                stmt_handle.add_diag_info(ODBCError::OptionValueChanged(
                    "SQL_ATTR_CONCURRENCY",
                    "SQL_CONCUR_READ_ONLY",
                ));
                SqlReturn::SUCCESS_WITH_INFO
            }
        },
        StatementAttribute::SQL_ATTR_CURSOR_TYPE => match FromPrimitive::from_i32(value_ptr as i32)
        {
            Some(CursorType::SQL_CURSOR_FORWARD_ONLY) => SqlReturn::SUCCESS,
            _ => {
                stmt_handle.add_diag_info(ODBCError::OptionValueChanged(
                    "SQL_ATTR_CURSOR_TYPE",
                    "SQL_CURSOR_FORWARD_ONLY",
                ));
                SqlReturn::SUCCESS_WITH_INFO
            }
        },
        StatementAttribute::SQL_ATTR_ENABLE_AUTO_IPD => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_ENABLE_AUTO_IPD"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_FETCH_BOOKMARK_PTR => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_FETCH_BOOKMARK_PTR"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_KEYSET_SIZE => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_KEYSET_SIZE"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_MAX_LENGTH => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_MAX_LENGTH"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_MAX_ROWS => {
            stmt.attributes.write().unwrap().max_rows = value_ptr as ULen;
            SqlReturn::SUCCESS
        }
        StatementAttribute::SQL_ATTR_NOSCAN => {
            match FromPrimitive::from_i32(value_ptr as i32) {
                Some(ns) => stmt.attributes.write().unwrap().no_scan = ns,
                None => stmt_handle.add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_NOSCAN")),
            }
            SqlReturn::SUCCESS
        }
        StatementAttribute::SQL_ATTR_PARAM_BIND_OFFSET_PTR => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_PARAM_BIND_OFFSET_PTR"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_PARAM_BIND_TYPE => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_PARAM_BIND_TYPE"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_PARAM_OPERATION_PTR => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_PARAM_OPERATION_PTR"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_PARAM_STATUS_PTR => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_PARAM_STATUS_PTR"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_PARAMS_PROCESSED_PTR => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_PARAMS_PROCESSED_PTR"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_PARAMSET_SIZE => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_PARAMSET_SIZE"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_QUERY_TIMEOUT => {
            stmt.attributes.write().unwrap().query_timeout = value_ptr as ULen;
            SqlReturn::SUCCESS
        }
        StatementAttribute::SQL_ATTR_RETRIEVE_DATA => {
            match FromPrimitive::from_i32(value_ptr as i32) {
                Some(RetrieveData::Off) => SqlReturn::SUCCESS,
                _ => {
                    stmt_handle
                        .add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_RETRIEVE_DATA"));
                    SqlReturn::ERROR
                }
            }
        }
        StatementAttribute::SQL_ATTR_ROW_BIND_OFFSET_PTR => {
            if !value_ptr.is_null(){
                add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("`column binding with offsets`"), "SQLSetStmtAttrW");
                return SqlReturn::ERROR;
            }

            stmt.attributes.write().unwrap().row_bind_offset_ptr = null_mut();
            SqlReturn::SUCCESS

        }
        StatementAttribute::SQL_ATTR_ROW_BIND_TYPE => {
            if value_ptr as ULen != BindType::SQL_BIND_BY_COLUMN as ULen{
                add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("`row-wise column binding`"), "SQLSetStmtAttrW");
                return SqlReturn::ERROR;
            }

            stmt.attributes.write().unwrap().row_bind_type = BindType::SQL_BIND_BY_COLUMN as ULen;
            SqlReturn::SUCCESS
        }
        StatementAttribute::SQL_ATTR_ROW_NUMBER => {
            stmt.attributes.write().unwrap().row_number = value_ptr as ULen;
            SqlReturn::SUCCESS
        }
        StatementAttribute::SQL_ATTR_ROW_OPERATION_PTR => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_ROW_OPERATION_PTR"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_ROW_STATUS_PTR => {
            stmt.attributes.write().unwrap().row_status_ptr = value_ptr.cast::<USmallInt>();
            SqlReturn::SUCCESS
        }
        StatementAttribute::SQL_ATTR_ROWS_FETCHED_PTR => {
            stmt.attributes.write().unwrap().rows_fetched_ptr = value_ptr.cast::<ULen>();
            SqlReturn::SUCCESS
        }
        StatementAttribute::SQL_ATTR_ROW_ARRAY_SIZE | StatementAttribute::SQL_ROWSET_SIZE => {
            match u32::try_from(value_ptr as ULen){
                Ok(ras) => {
                    stmt.attributes.write().unwrap().row_array_size = ras as ULen;
                    SqlReturn::SUCCESS
                },
                Err(_) => {
                    stmt.attributes.write().unwrap().row_array_size = u32::MAX as ULen;
                    add_diag_with_function!(stmt_handle, ODBCError::OptionValueChanged("SQL_ATTR_ROW_ARRAY_SIZE or SQL_ROWSET_SIZE", "4,294,967,295"), "SQLSetStmtAttrW");
                    SqlReturn::SUCCESS_WITH_INFO
                },
            }
        }
        StatementAttribute::SQL_ATTR_SIMULATE_CURSOR => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_SIMULATE_CURSOR"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_USE_BOOKMARKS => {
            match FromPrimitive::from_i32(value_ptr as i32) {
                Some(ub) => {
                    stmt.attributes.write().unwrap().use_bookmarks = ub;
                    SqlReturn::SUCCESS
                }
                None => {
                    stmt_handle
                        .add_diag_info(ODBCError::InvalidAttrValue("SQL_ATTR_USE_BOOKMARKS"));
                    SqlReturn::ERROR
                }
            }
        }
        StatementAttribute::SQL_ATTR_ASYNC_STMT_EVENT => {
            add_diag_with_function!(stmt_handle,ODBCError::Unimplemented("SQL_ATTR_ASYNC_STMT_EVENT"), "SQLSetStmtAttrW");
            SqlReturn::ERROR
        }
        StatementAttribute::SQL_ATTR_METADATA_ID => {
            todo!()
        }
        // leave SQL_GET_BOOKMARK as unsupported since it is for ODBC < 3.0 drivers
        StatementAttribute::SQL_GET_BOOKMARK
        // Not supported but still relevent to 3.0 drivers
        | StatementAttribute::SQL_ATTR_SAMPLE_SIZE
        | StatementAttribute::SQL_ATTR_DYNAMIC_COLUMNS
        | StatementAttribute::SQL_ATTR_TYPE_EXCEPTION_BEHAVIOR
        | StatementAttribute::SQL_ATTR_LENGTH_EXCEPTION_BEHAVIOR => {
            stmt_handle.add_diag_info(ODBCError::UnsupportedStatementAttribute(
                statement_attribute_to_string(attribute),
            ));
            SqlReturn::ERROR
        }
    }
}

///
/// [`SQLSpecialColumnsW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLSpecialColumns-function
///
/// This is the WideChar version of the SQLSpecialColumns function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLSpecialColumnsW(
    statement_handle: HStmt,
    _identifier_type: SmallInt,
    _catalog_name: *const WideChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WideChar,
    _schema_name_length: SmallInt,
    _table_name: *const WideChar,
    _table_name_length: SmallInt,
    _scope: SmallInt,
    _nullable: SmallInt,
) -> SqlReturn {
    unimpl!(statement_handle);
}

///
/// [`SQLStatistics`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLStatistics-function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLStatisticsW(
    statement_handle: HStmt,
    _catalog_name: *const WideChar,
    _catalog_name_length: SmallInt,
    _schema_name: *const WideChar,
    _schema_name_length: SmallInt,
    _table_name: *const WideChar,
    _table_name_length: SmallInt,
    _unique: SmallInt,
    _reserved: SmallInt,
) -> SqlReturn {
    unsupported_function!(statement_handle)
}

///
/// sql_stmt_close_cursor_helper is a helper function to ensure statements are disconnected
/// properly when a connection is disconnected or a statement is freed.
///
fn sql_stmt_close_cursor_helper(stmt: &Statement) {
    let _ = stmt.mongo_statement.write().map(|mut stmt| {
        stmt.as_mut().map(|stmt| {
            stmt.close_cursor();
        })
    });
}

///
/// [`SQLTablePrivilegesW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/sqltableprivileges-function
///
/// This is the WideChar version of the SQLTablesPrivileges function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[named]
#[no_mangle]
pub unsafe extern "C" fn SQLTablePrivilegesW(
    statement_handle: HStmt,
    _catalog_name: *const WideChar,
    _name_length_1: SmallInt,
    _schema_name: *const WideChar,
    _name_length_2: SmallInt,
    _table_name: *const WideChar,
    _name_length_3: SmallInt,
) -> SqlReturn {
    unimpl!(statement_handle);
}

#[allow(clippy::too_many_arguments)]
fn sql_tables(
    mongo_connection: &MongoConnection,
    query_timeout: i32,
    catalog: &str,
    schema: &str,
    table: &str,
    table_t: &str,
    odbc_3_behavior: bool,
    max_string_length: Option<u16>,
) -> Result<Box<dyn MongoStatement>> {
    match (catalog, schema, table, table_t) {
        (SQL_ALL_CATALOGS, "", "", "") => Ok(Box::new(MongoDatabases::list_all_catalogs(
            mongo_connection,
            Some(query_timeout),
        ))),
        ("", SQL_ALL_SCHEMAS, "", "") => {
            Ok(Box::new(MongoCollections::all_schemas(max_string_length)))
        }
        ("", "", "", SQL_ALL_TABLE_TYPES) => Ok(Box::new(MongoTableTypes::all_table_types())),
        _ => Ok(Box::new(MongoCollections::list_tables(
            mongo_connection,
            Some(query_timeout),
            catalog,
            table,
            table_t,
            odbc_3_behavior,
        ))),
    }
}

///
/// [`SQLTablesW`]: https://learn.microsoft.com/en-us/sql/odbc/reference/syntax/SQLTables-function
///
/// This is the WideChar version of the SQLTables function
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
#[no_mangle]
#[named]
pub unsafe extern "C" fn SQLTablesW(
    statement_handle: HStmt,
    catalog_name: *const WideChar,
    name_length_1: SmallInt,
    schema_name: *const WideChar,
    name_length_2: SmallInt,
    table_name: *const WideChar,
    name_length_3: SmallInt,
    table_type: *const WideChar,
    name_length_4: SmallInt,
) -> SqlReturn {
    panic_safe_exec_clear_diagnostics!(
        debug,
        || {
            let mongo_handle = try_mongo_handle!(statement_handle);
            let odbc_behavior = has_odbc_3_behavior!(mongo_handle);
            let stmt = must_be_valid!((*mongo_handle).as_statement());
            let catalog = input_text_to_string_w(catalog_name, name_length_1.into());
            let schema = input_text_to_string_w_allow_null(schema_name, name_length_2.into());
            let table = input_text_to_string_w_allow_null(table_name, name_length_3.into());
            let table_t = input_text_to_string_w(table_type, name_length_4.into());
            let connection = (*stmt.connection).as_connection().unwrap();
            let max_string_length = *connection.max_string_length.read().unwrap();
            let mongo_statement = sql_tables(
                connection
                    .mongo_connection
                    .read()
                    .unwrap()
                    .as_ref()
                    .unwrap(),
                stmt.attributes
                    .read()
                    .unwrap()
                    .query_timeout
                    .try_into()
                    .expect("Query timeout exceeds {i32::MAX}"),
                &catalog,
                &schema,
                &table,
                &table_t,
                odbc_behavior,
                max_string_length,
            );
            let mongo_statement = odbc_unwrap!(mongo_statement, mongo_handle);
            *stmt.mongo_statement.write().unwrap() = Some(mongo_statement);
            SqlReturn::SUCCESS
        },
        statement_handle
    );
}

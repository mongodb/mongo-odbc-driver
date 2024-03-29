use crate::{handles::definitions::*, SQLCancel};
use definitions::SqlReturn;
use mongo_odbc_core::MongoConnection;
// use mongodb::sync::Client;
use std::env;

mod integration {

    use super::*;

    fn generate_connection_uri() -> String {
        let user_name = env::var("ADF_TEST_LOCAL_USER").expect("ADF_TEST_LOCAL_USER is not set");
        let password = env::var("ADF_TEST_LOCAL_PWD").expect("ADF_TEST_LOCAL_PWD is not set");
        let host = env::var("ADF_TEST_LOCAL_HOST").expect("ADF_TEST_LOCAL_HOST is not set");
        format!("mongodb://{user_name}:{password}@{host}")
    }

    // skip-reason: SQL-1959
    // the local adf currently can't be configured to handle killop commands; it is on by default in production
    // this ticket will make killop configurable locally.
    // #[test]
    // fn test_cancel_running_query() {
    //     // allocate the handles
    //     let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
    //     let conn_handle = Connection::with_state(env, ConnectionState::Allocated);
    //     let mongo_connection = MongoConnection {
    //         client: Client::with_uri_str(generate_connection_uri()).unwrap(),
    //         operation_timeout: None,
    //         uuid_repr: None,
    //     };
    //     *conn_handle.mongo_connection.write().unwrap() = Some(mongo_connection);
    //     let conn = &mut MongoHandle::Connection(conn_handle);

    //     // let conn: &mut MongoHandle = &mut MongoHandle::Connection(create_connection());
    //     let stmt_handle = Statement::with_state(conn, StatementState::SynchronousQueryExecuting);
    //     let stmt_id = stmt_handle.statement_id.read().unwrap().clone();
    //     let stmt_id_ref = stmt_id.clone();
    //     let stmt: *mut _ = &mut MongoHandle::Statement(stmt_handle);

    //     unsafe {
    //         // we will create a new thread and make a new statement handle, setting its statement id to that of the original statement handle.
    //         // this will simulate how a multithreaded application could use a single statement, while respecting rusts borrowing rules.
    //         task::spawn(async move {
    //             let env_ref = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
    //             let conn_handle_ref = Connection::with_state(env_ref, ConnectionState::Allocated);
    //             let mongo_connection_ref = MongoConnection {
    //                 client: Client::with_uri_str(generate_connection_uri()).unwrap(),
    //                 operation_timeout: None,
    //                 uuid_repr: None,
    //             };
    //             *conn_handle_ref.mongo_connection.write().unwrap() = Some(mongo_connection_ref);
    //             conn_handle_ref.attributes.write().unwrap().current_catalog =
    //                 Some("tdvt".to_string());
    //             let conn_ref = &mut MongoHandle::Connection(conn_handle_ref);
    //             let stmt_handle_ref =
    //                 Statement::with_state(conn_ref, StatementState::SynchronousQueryExecuting);
    //             *stmt_handle_ref.statement_id.write().unwrap() = stmt_id_ref;
    //             let stmt_ref: *mut _ = &mut MongoHandle::Statement(stmt_handle_ref);

    //             // create a long running query and use SQLExecDirectW to execute it
    //             let mut query: Vec<WideChar> = cstr::to_widechar_vec("select * from batters where batters.Player in \
    //             ( select b2.Player from batters b2 where b2.BB / b2.AB > ( SELECT avg(b3.BB / b3.AB) from batters b3 ))");
    //             query.push(0);
    //             assert_eq!(
    //                 SQLExecDirectW(stmt_ref as *mut _, query.as_ptr(), query.len() as i32),
    //                 SqlReturn::ERROR
    //             );

    //             // when SQLCancel is called below, the long running query should be cancelled; verify we return the proper error in this case
    //             assert_eq!(
    //                 "[MongoDB][Core] Query was cancelled".to_string(),
    //                 format!(
    //                     "{}",
    //                     (*stmt_ref).as_statement().unwrap().errors.read().unwrap()[0]
    //                 ),
    //             );
    //         });

    //         // use SQLCancel to cancel the query
    //         thread::sleep(time::Duration::from_secs(5)); // ensure query has time to be issued before cancelling
    //         assert_eq!(SQLCancel(stmt as *mut _), SqlReturn::SUCCESS);

    //         // verify the query is no longer running by checking no current operations have the statement id as a comment
    //         thread::sleep(time::Duration::from_secs(5)); // ensure query has time to be cancelled before validating
    //         let mut cursor = conn
    //             .as_connection()
    //             .unwrap()
    //             .mongo_connection
    //             .read()
    //             .unwrap()
    //             .as_ref()
    //             .unwrap()
    //             .client
    //             .database("admin")
    //             .aggregate(vec![doc! {"$currentOp": {}}], None)
    //             .unwrap();

    //         assert!(!cursor.any(|row| {
    //             let operation = row.unwrap_or_default();
    //             if let Some(bson::Bson::Document(d)) = operation.get("command") {
    //                 d.get("comment") == Some(&stmt_id)
    //             } else {
    //                 false
    //             }
    //         }));
    //     }
    // }

    #[test]
    fn test_cancel_no_running_query() {
        let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
        let conn =
            &mut MongoHandle::Connection(Connection::with_state(env, ConnectionState::Allocated));

        // StatementState::Allocated is the only valid state other than the state indicating a query is executing.
        // SQLCancel should be a no op in this case; verify we get a success.
        let stmt: *mut _ =
            &mut MongoHandle::Statement(Statement::with_state(conn, StatementState::Allocated));
        unsafe { assert_eq!(SQLCancel(stmt as *mut _), SqlReturn::SUCCESS) }
    }

    // checks that cancel gracefully handles the case where a query was executing when cancel is called,
    // but is no longer executing when killop is ultimately issued
    // #[test]
    // fn test_cancel_running_query_not_executing() {
    //     let env = &mut MongoHandle::Env(Env::with_state(EnvState::Allocated));
    //     let conn_handle = Connection::with_state(env, ConnectionState::Allocated);
    //     let mongo_connection = MongoConnection {
    //         client: Client::with_uri_str(generate_connection_uri()).unwrap(),
    //         operation_timeout: None,
    //         uuid_repr: None,
    //     };
    //     *conn_handle.mongo_connection.write().unwrap() = Some(mongo_connection);
    //     let conn = &mut MongoHandle::Connection(conn_handle);

    //     let stmt_handle = Statement::with_state(conn, StatementState::SynchronousQueryExecuting);
    //     let stmt: *mut _ = &mut MongoHandle::Statement(stmt_handle);
    //     unsafe {
    //         assert_eq!(SQLCancel(stmt as *mut _), SqlReturn::SUCCESS);
    //     }
    // }
}

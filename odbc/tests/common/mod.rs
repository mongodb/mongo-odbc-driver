use atsql::SQLGetDiagRecW;
use odbc_sys::{Handle, HandleType};

// Verifies that the expected SQL State, message text, and native error in the handle match
// the expected input
pub fn verify_sql_diagnostics(
    handle_type: HandleType,
    handle: Handle,
    record_number: i16,
    expected_sql_state: &str,
    expected_message_text: &str,
    mut expected_native_err: i32,
) {
    use cstr::WideChar;
    let text_length_ptr = &mut 0;
    let mut actual_sql_state: [WideChar; 6] = [0; 6];
    let actual_sql_state = &mut actual_sql_state as *mut _;
    let mut actual_message_text: [WideChar; 512] = [0; 512];
    let actual_message_text = &mut actual_message_text as *mut _;
    let actual_native_error = &mut 0;
    unsafe {
        let _ = SQLGetDiagRecW(
            handle_type,
            handle as *mut _,
            record_number,
            actual_sql_state,
            actual_native_error,
            actual_message_text,
            1024,
            text_length_ptr,
        );
    };
    let mut expected_sql_state_encoded = cstr::to_widechar_vec(expected_sql_state);
    expected_sql_state_encoded.push(0);
    let actual_message_length = *text_length_ptr as usize;
    unsafe {
        assert_eq!(
            expected_message_text,
            &(cstr::from_widechar_ref_lossy(&*(actual_message_text as *const [u16; 256])))
                [0..actual_message_length],
        );
        assert_eq!(
            cstr::from_widechar_ref_lossy(
                &*(expected_sql_state_encoded.as_ptr() as *const [u16; 6])
            ),
            cstr::from_widechar_ref_lossy(&*(actual_sql_state as *const [u16; 6]))
        );
    }
    assert_eq!(&mut expected_native_err as &mut i32, actual_native_error);
}

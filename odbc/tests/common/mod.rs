use mongoodbc::SQLGetDiagRecW;
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
    let text_length_ptr = &mut 0;
    let actual_sql_state = &mut [0u16; 6] as *mut _;
    let actual_message_text = &mut [0u16; 512] as *mut _;
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
    let mut expected_sql_state_encoded: Vec<u16> = expected_sql_state.encode_utf16().collect();
    expected_sql_state_encoded.push(0);
    let actual_message_length = *text_length_ptr as usize;
    unsafe {
        assert_eq!(
            expected_message_text,
            &(String::from_utf16_lossy(&*(actual_message_text as *const [u16; 256])))
                [0..actual_message_length],
        );
        assert_eq!(
            String::from_utf16(&*(expected_sql_state_encoded.as_ptr() as *const [u16; 6])).unwrap(),
            String::from_utf16(&*(actual_sql_state as *const [u16; 6])).unwrap()
        );
    }
    assert_eq!(&mut expected_native_err as &mut i32, actual_native_error);
}

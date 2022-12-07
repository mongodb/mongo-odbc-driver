mod common;

mod integration {
    use odbc::*;
    use odbc_sys::{HDbc, InfoType, Pointer, SmallInt, SqlReturn, WChar};

    use std::ptr::copy_nonoverlapping;

    ///
    /// input_wtext_to_string converts an input cstring to a rust String.
    /// It assumes nul termination if the supplied length is negative.
    ///
    /// # Safety
    /// This converts raw C-pointers to rust Strings, which requires unsafe operations
    ///
    #[allow(clippy::uninit_vec)]
    pub unsafe fn input_wtext_to_string(text: *const WChar, len: usize) -> String {
        if (len as isize) < 0 {
            let mut dst = Vec::new();
            let mut itr = text;
            {
                while *itr != 0 {
                    dst.push(*itr);
                    itr = itr.offset(1);
                }
            }
            return String::from_utf16_lossy(&dst);
        }

        let mut dst = Vec::with_capacity(len);
        dst.set_len(len);
        copy_nonoverlapping(text, dst.as_mut_ptr(), len);
        String::from_utf16_lossy(&dst)
    }

    #[test]
    fn test_get_info_dbms_ver() {
        let env = create_environment_v3().unwrap();
        let conn_string = crate::common::generate_default_connection_str();

        let conn = env.connect_with_connection_string(&conn_string).unwrap();

        let value_ptr: *mut std::ffi::c_void = Box::into_raw(Box::new([0u8; 100])) as *mut _;
        let out_length: *mut SmallInt = &mut 10;

        unsafe {
            assert_eq!(
                SqlReturn::SUCCESS,
                odbc_sys::SQLGetInfoW(
                    conn.handle() as HDbc,
                    InfoType::DbmsVer,
                    value_ptr as Pointer,
                    11,
                    out_length,
                )
            );

            assert_eq!(10, *out_length);
            assert_eq!(
                "00.00.0000",
                input_wtext_to_string(value_ptr as *const _, *out_length as usize)
            );
        }
    }
}

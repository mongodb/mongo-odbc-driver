use std::ptr::copy_nonoverlapping;

pub type WideChar = u16;
pub type Char = u8;

pub fn from_widechar_vec_lossy(v: Vec<u16>) -> String {
    widestring::decode_utf16_lossy(v).collect::<String>()
}

pub fn from_widechar_ref_lossy(v: &[u16]) -> String {
    widestring::decode_utf16_lossy(v.iter().copied()).collect::<String>()
}

pub fn to_widechar_vec(s: &str) -> Vec<WideChar> {
    widestring::encode_utf16(s.chars()).collect::<Vec<_>>()
}

/// Converts a c char string to a rust string.
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
pub unsafe fn parse_string_a(str: *mut odbc_sys::Char) -> Option<String> {
    let string = unsafe { input_text_to_string(str, 1024) };
    match string.split_once(char::from(0)) {
        Some((string, _)) => Some(string.to_string()),
        _ => None,
    }
}

/// Converts a c wide char string to a rust string.
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
pub unsafe fn parse_string_w(str: *mut WideChar) -> Option<String> {
    let string = unsafe { input_wtext_to_string(str, 1024) };
    match string.split_once(char::from(0)) {
        Some((string, _)) => Some(string.to_string()),
        _ => None,
    }
}

///
/// input_text_to_string converts an input cstring to a rust String.
/// It assumes nul termination if the supplied length is negative.
///
/// # Safety
/// This converts raw C-pointers to rust Strings, which requires unsafe operations
///
#[allow(clippy::uninit_vec)]
pub unsafe fn input_text_to_string(text: *const Char, len: usize) -> String {
    if (len as isize) < 0 {
        let mut dst = Vec::new();
        let mut itr = text;
        {
            while *itr != 0 {
                dst.push(*itr);
                itr = itr.offset(1);
            }
        }
        return String::from_utf8_unchecked(dst);
    }

    let mut dst = Vec::with_capacity(len);
    dst.set_len(len);
    copy_nonoverlapping(text, dst.as_mut_ptr(), len);
    String::from_utf8_unchecked(dst)
}

///
/// input_wtext_to_string converts an input cstring to a rust String.
/// It assumes nul termination if the supplied length is negative.
///
/// # Safety
/// This converts raw C-pointers to rust Strings, which requires unsafe operations
///
#[allow(clippy::uninit_vec)]
pub unsafe fn input_wtext_to_string(text: *const WideChar, len: usize) -> String {
    if (len as isize) < 0 {
        let mut dst = Vec::new();
        let mut itr = text;
        {
            while *itr != 0 {
                dst.push(*itr);
                itr = itr.offset(1);
            }
        }
        return from_widechar_vec_lossy(dst);
    }

    let mut dst = Vec::with_capacity(len);
    dst.set_len(len);
    copy_nonoverlapping(text, dst.as_mut_ptr(), len);
    from_widechar_vec_lossy(dst)
}

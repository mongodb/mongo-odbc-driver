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
///
/// This function will attempt to read in up to 1024 characters.
///
/// The maximum length value size the registy allows is 16,383, but attempting to do this
/// results in crashes.
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
pub unsafe fn parse_registry_string_a(str: *mut odbc_sys::Char) -> Option<String> {
    let string = unsafe { input_text_to_string_a(str, 1024) };
    match string.split_once(char::from(0)) {
        Some((string, _)) => Some(string.to_string()),
        _ => None,
    }
}

/// Converts a c wide char string to a rust string.
///
/// This function will attempt to read in up to 1024 characters.
///
/// The maximum length value size the registry allows is 16,383, but attempting to do this
/// results in crashes.
///
/// # Safety
/// Because this is a C-interface, this is necessarily unsafe
///
pub unsafe fn parse_registry_string_w(str: *mut WideChar) -> Option<String> {
    let string = unsafe { input_text_to_string_w(str, 1024) };
    match string.split_once(char::from(0)) {
        Some((string, _)) => Some(string.to_string()),
        _ => None,
    }
}

///
/// input_text_to_string_a converts a u8 cstring to a rust String.
/// It assumes nul termination if the supplied length is negative.
///
/// # Safety
/// This converts raw C-pointers to rust Strings, which requires unsafe operations
///
#[allow(clippy::uninit_vec)]
pub unsafe fn input_text_to_string_a(text: *const Char, len: usize) -> String {
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
/// input_wtext_to_string converts a u16 cstring to a rust String.
/// It assumes nul termination if the supplied length is negative.
///
/// # Safety
/// This converts raw C-pointers to rust Strings, which requires unsafe operations
///
#[allow(clippy::uninit_vec)]
pub unsafe fn input_text_to_string_w(text: *const WideChar, len: usize) -> String {
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

pub fn to_widechar_ptr(s: &str) -> (*mut WideChar, Vec<u16>) {
    let mut v = to_widechar_vec(s);
    v.push(0);
    (v.as_mut_ptr(), v)
}

pub fn to_char_ptr(s: &str) -> (*mut Char, Vec<u8>) {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    (v.as_mut_ptr(), v)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_input_atext_to_string() {
        let expected = "test";
        let test = expected.as_bytes();
        let test = test.as_ptr();
        let test = unsafe { input_text_to_string_a(test, expected.len()) };
        assert_eq!(expected, test);
    }

    #[test]
    fn test_input_wtext_to_srtring() {
        let expected = "test";
        let test = to_widechar_vec(expected);
        let test = test.as_ptr();
        let test = unsafe { input_text_to_string_w(test, expected.len()) };
        assert_eq!(expected, test);
    }

    #[test]
    fn test_parse_registry_string_a() {
        let expected = "test";
        let mut test = Vec::from(expected.as_bytes());
        test.push(0);
        let test = test.as_mut_ptr() as *mut Char;
        let test = unsafe { parse_registry_string_a(test) };
        assert_eq!(expected, test.unwrap());
    }

    #[test]
    fn test_parse_string_w() {
        let expected = "test";
        let mut test = to_widechar_vec(expected);
        test.push(0);
        let test = test.as_mut_ptr() as *mut WideChar;
        let test = unsafe { parse_registry_string_w(test) };
        assert_eq!(expected, test.unwrap());
    }
}

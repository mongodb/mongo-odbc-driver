use num_derive::FromPrimitive;
use std::ptr::copy_nonoverlapping;

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum Charset {
    Utf16 = 1,
    Utf32 = 2,
}

pub type Char = u8;

#[cfg(feature = "utf32")]
pub type WideChar = u32;

#[cfg(feature = "utf32")]
pub const CHARSET: Charset = Charset::Utf32;
#[cfg(feature = "utf32")]
pub fn from_widechar_ref_lossy(v: &[WideChar]) -> String {
    widestring::decode_utf32_lossy(v.iter().copied()).collect::<String>()
}

#[cfg(feature = "utf32")]
pub fn from_widechar_vec_lossy(v: Vec<WideChar>) -> String {
    widestring::decode_utf32_lossy(v).collect::<String>()
}

#[cfg(feature = "utf32")]
pub fn to_widechar_vec(s: &str) -> Vec<WideChar> {
    widestring::encode_utf32(s.chars()).collect::<Vec<_>>()
}

#[cfg(not(feature = "utf32"))]
pub type WideChar = u16;

#[cfg(not(feature = "utf32"))]
pub const CHARSET: Charset = Charset::Utf16;

#[cfg(not(feature = "utf32"))]
pub fn from_widechar_vec_lossy(v: Vec<u16>) -> String {
    widestring::decode_utf16_lossy(v).collect::<String>()
}

#[cfg(not(feature = "utf32"))]
pub fn from_widechar_ref_lossy(v: &[WideChar]) -> String {
    widestring::decode_utf16_lossy(v.iter().copied()).collect::<String>()
}

#[cfg(not(feature = "utf32"))]
pub fn to_widechar_vec(s: &str) -> Vec<WideChar> {
    widestring::encode_utf16(s.chars()).collect::<Vec<_>>()
}

///
/// input_text_to_string_a converts a u8 cstring to a rust String.
/// It assumes null termination if the supplied length is negative.
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
/// input_text_to_string_w converts a u16 cstring to a rust String.
/// It assumes null termination if the supplied length is negative.
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
///
/// parse_attribute_string_w converts a null-separated doubly null terminated *Widechar string to a Rust
/// string separated by `;`.
///
/// # Safety
/// This converts a raw c-pointer to a Rust string, which requires unsafe operations. Additionally, it
/// has the small possibility of reading into unallocated memory should the input string not be doubly
/// null terminated. Only use this method if you are certain the input string is doubly null terminated.
///
#[allow(clippy::uninit_vec)]
pub unsafe fn parse_attribute_string_w(text: *const WideChar) -> String {
    let mut dst = Vec::new();
    let mut itr = text;
    {
        while *itr != 0 || *itr.offset(1) != 0 {
            dst.push(*itr);
            itr = itr.offset(1);
        }
    }
    from_widechar_vec_lossy(dst).replace(char::from(0), ";")
}

///
/// parse_attribute_string_a converts a null-separated doubly null terminated *Char string to a Rust
/// string separated by `;`.
///
/// # Safety
/// This converts a raw c-pointer to a Rust string, which requires unsafe operations. Additionally, it
/// has the small possibility of reading into unallocated memory should the input string not be doubly
/// null terminated. Only use this method if you are certain the input string is doubly null terminated.
///
#[allow(clippy::uninit_vec)]
pub unsafe fn parse_attribute_string_a(text: *const Char) -> String {
    let mut dst = Vec::new();
    let mut itr = text;
    {
        while *itr != 0 || *itr.offset(1) != 0 {
            dst.push(*itr);
            itr = itr.offset(1);
        }
    }
    String::from_utf8(dst).unwrap().replace(char::from(0), ";")
}

///
/// to_widechar_ptr converts the input string to a null terminated string encoded in UTF-16.
///
pub fn to_widechar_ptr(s: &str) -> (*mut WideChar, Vec<WideChar>) {
    let mut v = to_widechar_vec(s);
    v.push(0);
    (v.as_mut_ptr(), v)
}

///
/// to_char_ptr converts the input string to a null terminated string encoded in UTF-8.
///
pub fn to_char_ptr(s: &str) -> (*mut Char, Vec<u8>) {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    (v.as_mut_ptr(), v)
}

///
/// write_widechar_to_buffer writes the input string to the output buffer, and returns the number of bytes written
///
/// # Safety
/// This writes to a raw c-pointer, which requires unsafe operations
pub unsafe fn write_widechar_to_buffer(
    message: &str,
    len: usize,
    output_ptr: *mut WideChar,
) -> u16 {
    let len = std::cmp::min(message.len(), len - 1);
    let mut v = to_widechar_vec(&message[..len]);
    v.push(0);
    unsafe {
        copy_nonoverlapping(v.as_mut_ptr(), output_ptr, len);
    }
    v.len() as u16
}

///
/// write_char_to_buffer writes the input string to the output buffer, and returns the number of bytes written
///
/// # Safety
/// This writes to a raw c-pointer, which requires unsafe operations
pub unsafe fn write_char_to_buffer(message: &str, len: usize, output_ptr: *mut Char) -> u16 {
    let len = std::cmp::min(message.len(), len - 1);

    let chars_to_write = &mut message[..len].to_string();

    let v = chars_to_write.as_mut_vec();
    v.push(0);
    unsafe {
        copy_nonoverlapping(v.as_mut_ptr(), output_ptr, len);
    }
    v.len() as u16
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_input_atext_to_string() {
        let expected = "test";
        let test = "test\0".as_bytes();
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
    fn test_write_to_buffer_with_enough_space() {
        let expected = "test\0\0\0\0\0";
        let input = "test";
        let mut buffer = [0; 9];
        let len = unsafe { write_widechar_to_buffer(input, buffer.len(), buffer.as_mut_ptr()) };
        assert_eq!(expected, from_widechar_ref_lossy(&buffer));
        assert_eq!(len, std::cmp::min(input.len() + 1, buffer.len()) as u16);
    }

    #[test]
    fn test_write_to_buffer_constrained_space() {
        let expected = "te\0";
        let input = "testing";
        let mut buffer: [WideChar; 3] = [0; 3];
        let len = unsafe { write_widechar_to_buffer("testing", buffer.len(), buffer.as_mut_ptr()) };
        assert_eq!(expected, from_widechar_ref_lossy(&buffer));
        assert_eq!(len, std::cmp::min(input.len() + 1, buffer.len()) as u16);
    }

    #[test]
    fn test_write_to_buffer_when_message_len_is_buffer_len() {
        let expected = "tes\0";
        let input = "test";
        let mut buffer: [WideChar; 4] = [0; 4];
        let len = unsafe { write_widechar_to_buffer(input, buffer.len(), buffer.as_mut_ptr()) };
        assert_eq!(expected, from_widechar_ref_lossy(&buffer));
        assert_eq!(len, std::cmp::min(input.len() + 1, buffer.len()) as u16);
    }
}

pub type WideChar = u16;

pub fn from_widechar_vec_lossy(v: Vec<u16>) -> String {
    widestring::decode_utf16_lossy(v).collect::<String>()
}

pub fn from_widechar_ref_lossy(v: &[u16]) -> String {
    widestring::decode_utf16_lossy(v.iter().copied()).collect::<String>()
}

pub fn to_widechar_vec(s: &str) -> Vec<WideChar> {
    widestring::encode_utf16(s.chars()).collect::<Vec<_>>()
}

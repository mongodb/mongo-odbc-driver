use bson::{to_vec, Document};
use std::mem::forget;

#[repr(C)]
pub struct BsonResult {
    ptr: *const u8,
    len: usize,
    cap: usize,
}

/// # Safety
/// The caller must ensure that the `command` is a valid pointer to a UTF-8 byte slice.
#[no_mangle]
pub unsafe extern "C" fn runCommand(command: *const u8, length: usize) -> BsonResult {
    let bson_bytes_slice = std::slice::from_raw_parts(command, length);
    let result_docs: Document = bson::from_slice(bson_bytes_slice).unwrap();

    let bytes = to_vec(&result_docs).expect("Failed to convert to BSON");
    let ptr = bytes.as_ptr();
    let len = bytes.len();
    let cap = bytes.capacity();

    forget(bytes);

    BsonResult { ptr, len, cap }
}

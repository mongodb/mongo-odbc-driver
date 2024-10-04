use bson::{to_vec, Document};
use std::mem::forget;

// This mock library is designed to simulate the behavior of the `mongosqltranslate` library
// for testing purposes. It provides a simple implementation of the `runCommand` function.
#[repr(C)]
pub struct BsonBuffer {
    ptr: *const u8,
    len: usize,
    cap: usize,
}

/// # Safety
/// The caller must ensure that the `command.ptr` is a valid pointer to a UTF-8 byte slice.
#[no_mangle]
pub unsafe extern "C" fn runCommand(command: BsonBuffer) -> BsonBuffer {
    let bson_bytes_slice = Vec::from_raw_parts(command.ptr.cast_mut(), command.len, command.cap);
    let result_docs: Document = bson::from_slice(&bson_bytes_slice).unwrap();

    let bytes = to_vec(&result_docs).expect("Failed to convert to BSON");
    let ptr = bytes.as_ptr();
    let len = bytes.len();
    let cap = bytes.capacity();

    BsonBuffer { ptr, len, cap }
}

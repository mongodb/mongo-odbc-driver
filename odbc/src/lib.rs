mod api;
mod handles;

pub use api::*;

#[cfg(target_os = "windows")]
use windows::{Win32::Foundation::*, Win32::System::LibraryLoader::*};

#[cfg(target_os = "windows")]
use std::mem::forget;

#[cfg(target_os = "windows")]
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => forget(LoadLibraryA("user32.dll")),
        DLL_PROCESS_DETACH => (),
        _ => (),
    }

    true
}

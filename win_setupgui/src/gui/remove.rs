#[cfg(target_os = "windows")]
extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use mongo_odbc_core::util::dsn::windows::DSNOpts;
use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct RemoveGui {
    #[nwg_control(size: (300, 135), position: (300, 300), title: "AtlasSQL ODBC Driver Source Configuration", flags: "WINDOW|DISABLED")]
    #[nwg_events( OnWindowClose: [RemoveGui::close] )]
    window: nwg::Window,
}

impl RemoveGui {
    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
}

pub fn remove_dsn(opts: DSNOpts) {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let app = RemoveGui::build_ui(Default::default()).expect("Failed to build UI");
    match opts.remove_from_registry() {
        Ok(_) => {
            nwg::modal_info_message(
                &app.window,
                "Removed",
                &format!("DSN {} has been removed.", &opts.dsn),
            );
        }
        Err(e) => {
            nwg::modal_error_message(
                &app.window,
                "Error",
                &format!("Failed to remove DSN: {}. Error: {}", &opts.dsn, e),
            );
        }
    }
}

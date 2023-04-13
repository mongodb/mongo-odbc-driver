extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use cstr::{input_text_to_string_w, to_widechar_ptr};
use nwd::NwgUi;
use nwg::NativeUi;
use shared_sql_utils::{Dsn, DsnArgs};
use std::{cell::RefCell, thread};
use windows::Win32::System::Search::{ODBC_ADD_DSN, ODBC_CONFIG_DSN};

#[link(name = "atsql", kind = "raw-dylib")]
extern "C" {
    /// atlas_sql_test_connection returns true if a connection can be established
    /// with the provided connection string.
    /// If the connection fails, the error message is written to the buffer.
    ///
    /// # Arguments
    /// * `connection_string` - A null-terminated widechar string containing the connection string.
    /// * `buffer` - A buffer to write the error message to, in widechar chars.
    /// * `buffer_in_len` - The length of the buffer, in widechar chars.
    /// * `buffer_out_length` - The length of data written to buffer, in widechar chars.
    ///
    /// # Safety
    /// Because this function is called from C, it is unsafe.
    ///
    fn atlas_sql_test_connection(
        connection_string: *const u16,
        buffer: *const u16,
        buffer_in_length: usize,
        buffer_out_length: *mut u16,
    ) -> bool;
}

#[derive(Default, NwgUi)]
pub struct ConfigGui {
    dialog_data: RefCell<Option<thread::JoinHandle<String>>>,

    #[nwg_control(size: (600, 450), position: (500, 500), title: "MongoDB Atlas SQL ODBC Driver DSN Configuration", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [ConfigGui::close_cancel] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 5)]
    grid: nwg::GridLayout,

    #[nwg_control(flags: "VISIBLE", text: "DSN")]
    #[nwg_layout_item(layout: grid, row: 1, col: 0, col_span: 2)]
    dsn_field: nwg::Label,

    #[nwg_control(flags: "VISIBLE", text: "", focus: true)]
    #[nwg_layout_item(layout: grid, row: 1, col: 2, col_span: 7)]
    dsn_input: nwg::TextInput,

    #[nwg_control(flags: "VISIBLE", text: "Username")]
    #[nwg_layout_item(layout: grid,  row: 2, col: 0, col_span: 2)]
    user_field: nwg::Label,

    #[nwg_control(flags: "VISIBLE", text: "")]
    #[nwg_layout_item(layout: grid,  row: 2, col: 2, col_span: 7)]
    user_input: nwg::TextInput,

    #[nwg_control(flags: "VISIBLE", text: "Password")]
    #[nwg_layout_item(layout: grid,  row: 3, col: 0, col_span: 2)]
    password_field: nwg::Label,

    #[nwg_control(flags: "VISIBLE", text: "", password: Some('*'))]
    #[nwg_layout_item(layout: grid,  row: 3, col: 2, col_span: 7)]
    password_input: nwg::TextInput,

    #[nwg_control(flags: "VISIBLE", text: "MongoDB URI")]
    #[nwg_layout_item(layout: grid,  row: 4, col: 0, col_span: 2)]
    mongodb_uri_field: nwg::Label,

    #[nwg_control(flags: "VISIBLE", text: "")]
    #[nwg_layout_item(layout: grid,  row: 4, col: 2, col_span: 7)]
    mongodb_uri_input: nwg::TextInput,

    #[nwg_control(flags: "VISIBLE", text: "Database")]
    #[nwg_layout_item(layout: grid,  row: 5, col: 0, col_span: 2)]
    database_field: nwg::Label,

    #[nwg_control(flags: "VISIBLE", text: "")]
    #[nwg_layout_item(layout: grid,  row: 5, col: 2, col_span: 7)]
    database_input: nwg::TextInput,

    #[nwg_control(flags: "VISIBLE", text: "Log Level")]
    #[nwg_layout_item(layout: grid,  row: 6, col: 0, col_span: 2)]
    log_level_field: nwg::Label,

    #[nwg_control(flags: "VISIBLE", text: "Info")]
    #[nwg_layout_item(layout: grid,  row: 6, col: 3, col_span: 6)]
    log_level_input: nwg::TextInput,

    #[nwg_control(flags: "VISIBLE", text: "Test")]
    #[nwg_events( OnButtonClick: [ConfigGui::test_connection] )]
    #[nwg_layout_item(layout: grid,  row: 10, col: 0, col_span: 1)]
    test_button: nwg::Button,

    #[nwg_control(flags: "VISIBLE", text: "Cancel")]
    #[nwg_events( OnButtonClick: [ConfigGui::close_cancel] )]
    #[nwg_layout_item(layout: grid,  row: 10, col: 6, col_span: 2)]
    close_button: nwg::Button,

    #[nwg_control(flags: "VISIBLE", text: "Ok")]
    #[nwg_events( OnButtonClick: [ConfigGui::open_confirm] )]
    #[nwg_layout_item(layout: grid,  row: 10, col: 8, col_span: 1)]
    ok_button: nwg::Button,

    #[nwg_control(flags: "DISABLED", text: "")]
    driver_name: nwg::Label,

    #[nwg_control()]
    #[nwg_events( OnNotice: [ConfigGui::read_confirm_output] )]
    dialog_notice: nwg::Notice,

    close: RefCell<bool>,

    operation: RefCell<(String, String)>,
}

impl ConfigGui {
    fn close_ok(&self) {
        *self.close.borrow_mut() = true;
        nwg::stop_thread_dispatch();
    }

    fn close_cancel(&self) {
        nwg::stop_thread_dispatch();
    }

    fn validate_input(&self) -> Option<Dsn> {
        match (
            self.database_input.text().is_empty(),
            self.dsn_input.text().is_empty(),
            self.password_input.text().is_empty(),
            self.mongodb_uri_input.text().is_empty(),
            self.user_input.text().is_empty(),
            self.log_level_input.text().is_empty(),
        ) {
            (false, false, false, false, false, false) => {}
            _ => {
                nwg::modal_error_message(
                    &self.window,
                    "Error",
                    "All fields are required and cannot be empty.",
                );
                return None;
            }
        }
        match self.log_level_input.text().to_lowercase().as_str() {
            "info" | "debug" | "error" => {}
            _ => {
                nwg::modal_error_message(
                    &self.window,
                    "Error",
                    "Log level must be one of: Info, Debug, Error.",
                );
                return None;
            }
        }
        match Dsn::new(DsnArgs {
            database: self.database_input.text().as_str(),
            dsn: self.dsn_input.text().as_str(),
            password: self.password_input.text().as_str(),
            uri: self.mongodb_uri_input.text().as_str(),
            user: self.user_input.text().as_str(),
            server: "",
            driver_name: self.driver_name.text().as_str(),
            log_level: self.log_level_input.text().as_str(),
        }) {
            Err(e) => {
                nwg::modal_error_message(&self.window, "Error", &e.to_string());
                None
            }
            Ok(opts) => Some(opts),
        }
    }

    fn test_connection(&self) {
        let mut buffer = vec![0u16; 1024];
        let mut buffer_length = 0u16;
        if let Some(opts) = self.validate_input() {
            if unsafe {
                atlas_sql_test_connection(
                    to_widechar_ptr(&opts.to_connection_string()).0 as *mut u16,
                    buffer.as_mut_ptr(),
                    buffer.len(),
                    &mut buffer_length,
                )
            } {
                nwg::modal_info_message(
                    &self.window,
                    "Success",
                    "Connected successfully with supplied information.",
                );
            } else {
                nwg::modal_error_message(
                    &self.window,
                    "Error",
                    &format!(
                        "Could not connect with supplied information: {e}",
                        e = unsafe {
                            input_text_to_string_w(buffer.as_mut_ptr(), buffer_length as usize)
                        }
                    ),
                );
            }
        }
    }

    fn set_keys(&self) {
        if let Some(dsn_opts) = self.validate_input() {
            match dsn_opts.write_dsn_to_registry() {
                true => {
                    nwg::modal_info_message(
                        &self.window,
                        "Success",
                        &format!(
                            "DSN {dsn} {verbed} successfully",
                            dsn = dsn_opts.dsn,
                            verbed = self.operation.borrow().1
                        ),
                    );
                    self.close_ok();
                }
                false => {
                    nwg::modal_error_message(
                        &self.window,
                        "Error",
                        &format!("Could not {verb} DSN", verb = self.operation.borrow().0),
                    );
                }
            }
        }
    }
    // SQL-1281
    // fn open_dir_picker(&self) {
    //     if let Ok(d) = std::env::current_dir() {
    //         if let Some(d) = d.to_str() {
    //             self.dialog
    //                 .set_default_folder(d)
    //                 .expect("Failed to set default folder.");
    //         }
    //     }

    //     if self.dialog.run(Some(&self.window)) {
    //         self.log_path_input.set_text("");
    //         if let Ok(directory) = self.dialog.get_selected_item() {
    //             let dir = directory.into_string().unwrap();
    //             self.log_path_input.set_text(&dir);
    //         }
    //     }
    // }

    fn open_confirm(&self) {
        // Disable the button to stop the user from spawning multiple dialogs
        self.ok_button.set_enabled(false);

        *self.dialog_data.borrow_mut() = Some(ConfirmCancelConfigDialog::popup(
            self.dialog_notice.sender(),
        ));
    }

    fn read_confirm_output(&self) {
        self.ok_button.set_enabled(true);

        let data = self.dialog_data.borrow_mut().take();
        if let Some(handle) = data {
            let dialog_result = handle.join().unwrap();
            if dialog_result == "Confirm" {
                self.set_keys();
            }
        }
    }
}

pub fn config_dsn(dsn_opts: Dsn, dsn_op: u32) -> bool {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let app = ConfigGui::build_ui(ConfigGui {
        ..Default::default()
    })
    .expect("Failed to build UI");

    match dsn_op {
        ODBC_ADD_DSN => {
            *app.operation.borrow_mut() = (String::from("add"), String::from("added"));
        }
        ODBC_CONFIG_DSN => {
            *app.operation.borrow_mut() = (String::from("configure"), String::from("configured"));
            app.dsn_input.set_text(&dsn_opts.dsn);
            app.dsn_input.set_readonly(true);
            app.database_input.set_text(&dsn_opts.database);
            app.mongodb_uri_input.set_text(&dsn_opts.uri);
            // SQL-1281
            // app.log_path_input.set_text(&dsn_opts.logpath);
            app.user_input.set_text(&dsn_opts.user);
            app.password_input.set_text(&dsn_opts.password);
        }
        _ => unreachable!(),
    }

    app.driver_name.set_text(&dsn_opts.driver_name);
    // SQL-1281
    // app.directory_button
    //     .set_text(String::from_utf16(&[0xD83D, 0xDCC1]).unwrap().as_str());
    nwg::dispatch_thread_events();
    app.close.take()
}

#[derive(Default, NwgUi)]
pub struct ConfirmCancelConfigDialog {
    data: RefCell<Option<String>>,

    #[nwg_control(size: (300, 150), position: (650, 300), title: "Confirm", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [ConfirmCancelConfigDialog::close] )]
    window: nwg::Window,

    #[nwg_control(flags: "VISIBLE", position: (10, 10), size: (280, 50), text: "Confirm saving changes?")]
    message: nwg::Label,

    #[nwg_control(flags: "VISIBLE", text: "Cancel", position: (100, 90), size: (80, 35))]
    #[nwg_events( OnButtonClick: [ConfirmCancelConfigDialog::choose(SELF, CTRL)] )]
    choice_no: nwg::Button,

    #[nwg_control(flags: "VISIBLE", text: "Confirm", focus: true, position: (200, 90), size: (80, 35))]
    #[nwg_events( OnButtonClick: [ConfirmCancelConfigDialog::choose(SELF, CTRL)] )]
    choice_yes: nwg::Button,
}

impl ConfirmCancelConfigDialog {
    /// Create the dialog UI on a new thread. The dialog result will be returned by the thread handle.
    /// To alert the main GUI that the dialog completed, this function takes a notice sender object.
    fn popup(sender: nwg::NoticeSender) -> thread::JoinHandle<String> {
        thread::spawn(move || {
            // Create the UI just like in the main function
            let app = ConfirmCancelConfigDialog::build_ui(Default::default())
                .expect("Failed to build UI");
            nwg::dispatch_thread_events();

            // Notice the main thread that the dialog completed
            sender.notice();

            // Return the dialog data
            app.data.take().unwrap_or("Cancel".to_owned())
        })
    }

    fn close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn choose(&self, btn: &nwg::Button) {
        let mut data = self.data.borrow_mut();
        if btn == &self.choice_no {
            *data = Some("Cancel".to_string());
        } else if btn == &self.choice_yes {
            *data = Some("Confirm".to_string());
        }

        self.window.close();
    }
}

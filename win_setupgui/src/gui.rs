extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use mongo_odbc_core::util::dsn::DSNOpts;
use nwd::NwgUi;
use nwg::NativeUi;
use std::{cell::RefCell, thread};
use windows::Win32::System::Search::{ODBC_ADD_DSN, ODBC_CONFIG_DSN};

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

    // #[nwg_control(flags: "VISIBLE", text: "Log Path")]
    // #[nwg_layout_item(layout: grid,  row: 6, col: 0, col_span: 2)]
    // log_path_field: nwg::Label,

    // #[nwg_resource(title: "Select Directory", action: nwg::FileDialogAction::OpenDirectory)]
    // dialog: nwg::FileDialog,

    // #[nwg_control(flags: "VISIBLE")]
    // #[nwg_layout_item(layout: grid,  row: 6, col: 2, col_span: 1)]
    // #[nwg_events( OnButtonClick: [ConfigGui::open_dir_picker] )]
    // directory_button: nwg::Button,

    // #[nwg_control(flags: "VISIBLE", text: "")]
    // #[nwg_layout_item(layout: grid,  row: 6, col: 3, col_span: 6)]
    // log_path_input: nwg::TextInput,
    #[nwg_control(flags: "VISIBLE", text: "Test")]
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

    gui_opts: GuiOpts,

    close: RefCell<bool>,
}

impl ConfigGui {
    fn close_ok(&self) {
        *self.close.borrow_mut() = true;
        nwg::stop_thread_dispatch();
    }

    fn close_cancel(&self) {
        nwg::stop_thread_dispatch();
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

    fn set_keys(&self) {
        let dsn_opts = DSNOpts {
            dsn: self.dsn_input.text(),
            database: self.database_input.text(),
            server: self.mongodb_uri_input.text(),
            user: self.user_input.text(),
            password: self.password_input.text(),
            // SQL-1281
            // logpath: self.log_path_input.text(),
            driver_name: self.driver_name.text(),
        };
        match self.gui_opts.op == ODBC_CONFIG_DSN
            || (self.gui_opts.op == ODBC_ADD_DSN && dsn_opts.is_valid_dsn())
        {
            false => {
                nwg::modal_error_message(
                    &self.window,
                    "Error",
                    &format!("Invalid DSN: {dsn}\nDSN may not be longer than 32 characters, and may not contain any of the following characters: [ ] {{ }} ( ) , ; ? * = ! @ \\", dsn = dsn_opts.dsn),
                );
            }
            true => match dsn_opts.write_dsn_to_registry() {
                true => {
                    nwg::modal_info_message(
                        &self.window,
                        "Success",
                        &format!(
                            "DSN {dsn} {verbed} successfully",
                            dsn = dsn_opts.dsn,
                            verbed = self.gui_opts.verbed
                        ),
                    );
                    self.close_ok();
                }
                false => {
                    nwg::modal_error_message(
                        &self.window,
                        "Error",
                        &format!("Could not {verb} DSN", verb = self.gui_opts.verb),
                    );
                }
            },
        }
    }
}

pub fn config_dsn(dsn_opts: DSNOpts, dsn_op: u32) -> bool {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let gui_opts = GuiOpts::new(dsn_op);
    let app = ConfigGui::build_ui(ConfigGui {
        gui_opts,
        ..Default::default()
    })
    .expect("Failed to build UI");

    match dsn_op {
        ODBC_ADD_DSN => {}
        ODBC_CONFIG_DSN => {
            app.dsn_input.set_text(&dsn_opts.dsn);
            app.dsn_input.set_readonly(true);
            app.database_input.set_text(&dsn_opts.database);
            app.mongodb_uri_input.set_text(&dsn_opts.server);
            // SQL-1281
            // app.log_path_input.set_text(&dsn_opts.logpath);
            app.user_input.set_text(&dsn_opts.user);
            app.password_input.set_text(&dsn_opts.password);
        }
        _ => unreachable!(),
    }

    app.driver_name.set_text(&dsn_opts.driver_name);
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

#[derive(Default, Debug)]
struct GuiOpts {
    op: u32,
    verb: String,
    verbed: String,
}

impl GuiOpts {
    fn new(op: u32) -> Self {
        match op {
            ODBC_ADD_DSN => GuiOpts {
                op: ODBC_ADD_DSN,
                verb: "add".to_string(),
                verbed: "added".to_string(),
            },
            ODBC_CONFIG_DSN => GuiOpts {
                op: ODBC_CONFIG_DSN,
                verb: "configure".to_string(),
                verbed: "configured".to_string(),
            },
            _ => unreachable!(),
        }
    }
}

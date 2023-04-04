use constants::{DRIVER_MAJOR_MINOR_VERSION, DRIVER_NAME_INSTALLED_VERSION};
use cstr::to_widechar_ptr;
use log::LevelFilter;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Handle;
use log4rs::{
    append::rolling_file::{
        policy::compound::{
            roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
        },
        RollingFileAppender,
    },
    config::{Appender, Config, Root},
};
use shared_sql_utils::odbcinst::SQLGetPrivateProfileStringW;
use std::path::{Path, PathBuf};

const LOG_FILE_SIZE: u64 = 1024 * 500;

#[derive(Debug)]
pub struct Logger {
    handle: Handle,
    log_dir: PathBuf,
}

impl Logger {
    pub fn new() -> Option<Self> {
        // Due to numerous reasons why the logger could fail to initialize, we wrap it in a catch_unwind
        // so that logger failure does not cause our dll to crash.
        match std::panic::catch_unwind(|| {
            let mut buffer = [0u16; 1024];
            unsafe {
                SQLGetPrivateProfileStringW(
                    to_widechar_ptr(DRIVER_NAME_INSTALLED_VERSION.as_str()).0,
                    to_widechar_ptr("Driver").0,
                    to_widechar_ptr("").0,
                    buffer.as_mut_ptr(),
                    buffer.len() as i32,
                    to_widechar_ptr("odbcinst.ini").0,
                )
            };
            let driver_path = unsafe { cstr::parse_attribute_string_w(buffer.as_mut_ptr()) };
            dbg!(&driver_path);

            let path = Path::new(&driver_path);
            let parent = path.parent().unwrap();
            let log_dir = parent.parent().unwrap().join("log");
            let logfile = Self::file_appender(log_dir.to_str().unwrap());
            let handle = Self::init_logger(logfile);

            Self { handle, log_dir }
        }) {
            Ok(logger) => Some(logger),
            Err(_) => None,
        }
    }

    pub fn set_log_level(&self, level_filter: String) {
        let level_filter = match level_filter.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        };

        let config = Config::builder()
            .appender(Appender::builder().build(
                "logfile",
                Box::new(Logger::file_appender(self.log_dir.to_str().unwrap())),
            ))
            .build(Root::builder().appender("logfile").build(level_filter))
            .unwrap();
        self.handle.set_config(config);
    }

    fn file_appender(log_dir: &str) -> RollingFileAppender {
        let file_path = format!(
            "{log_dir}/mongo_odbc-{}.log",
            DRIVER_MAJOR_MINOR_VERSION.as_str()
        );
        let roller_pattern = format!(
            "{log_dir}/mongo_odbc-{}.log.{{}}",
            DRIVER_MAJOR_MINOR_VERSION.as_str()
        );

        let roller = FixedWindowRoller::builder()
            .build(&roller_pattern, 10)
            .unwrap();
        let trigger = SizeTrigger::new(LOG_FILE_SIZE);
        let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));

        RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{d(%Y-%m-%d %H:%M:%S)(utc)} - {h({l})}: {m}{n}",
            )))
            .append(true)
            .build(file_path, Box::new(policy))
            .unwrap()
    }

    fn init_logger(logfile: RollingFileAppender) -> Handle {
        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(LevelFilter::Info))
            .unwrap();
        log4rs::init_config(config).unwrap()
    }
}

#[cfg(test)]
mod driver {

    use super::*;
    use log::{debug, error, info};

    #[test]
    fn logger() {
        let logger = Logger::new();
        info!("info1");
        debug!("debug1");
        error!("error1");
        logger.as_ref().unwrap().set_log_level("debug".to_string());
        info!("info2");
        debug!("debug2");
        error!("error2");
        logger.as_ref().unwrap().set_log_level("error".to_string());
        info!("info3");
        debug!("debug3");
        error!("error3");
    }
}

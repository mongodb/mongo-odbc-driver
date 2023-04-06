use constants::DRIVER_NAME;
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
                    to_widechar_ptr(DRIVER_NAME).0,
                    to_widechar_ptr("Driver").0,
                    to_widechar_ptr("").0,
                    buffer.as_mut_ptr(),
                    buffer.len() as i32,
                    to_widechar_ptr("odbcinst.ini").0,
                )
            };

            let driver_path = unsafe { cstr::parse_attribute_string_w(buffer.as_mut_ptr()) };

            let log_dir = if driver_path.is_empty() {
                std::env::temp_dir()
            } else {
                let path = Path::new(&driver_path);
                path.parent()
                    .map(|p| p.parent().map(|p| p.join("logs")).unwrap())
                    .unwrap_or_else(std::env::temp_dir)
            };

            if let Some(log_dir_str) = log_dir.to_str() {
                if let Ok(appender) = Self::file_appender(log_dir_str) {
                    match Self::init_logger(appender) {
                        Ok(handle) => Some(Self { handle, log_dir }),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            Ok(logger) => logger,
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

        if let Some(log_dir) = self.log_dir.to_str() {
            if let Ok(appender) = Logger::file_appender(log_dir) {
                let config = Config::builder()
                    .appender(Appender::builder().build("logfile", Box::new(appender)))
                    .build(Root::builder().appender("logfile").build(level_filter))
                    .unwrap();
                self.handle.set_config(config);
            }
        }
    }

    fn file_appender(log_dir: &str) -> Result<RollingFileAppender, std::io::Error> {
        let file_path = format!("{log_dir}/mongo_odbc.log");
        let roller_pattern = format!("{log_dir}/mongo_odbc.log.{{}}");

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
    }

    fn init_logger(logfile: RollingFileAppender) -> Result<Handle, ()> {
        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(LevelFilter::Info))
            .map_err(|_e| ())?;
        log4rs::init_config(config).map_err(|_e| ())
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

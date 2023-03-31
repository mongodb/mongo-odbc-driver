use cstr::*;
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
use shared_sql_utils::odbcinst::SQLGetPrivateProfileString;
use std::path::{Path, PathBuf};

// 50KB
const LOG_FILE_SIZE: u64 = 1024 * 1024 * 50;

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
            let mut buffer = [0u8; 1024];
            unsafe {
                SQLGetPrivateProfileString(
                    to_char_ptr("MongoDB Atlas SQL ODBC Driver").0,
                    to_char_ptr("Driver").0,
                    to_char_ptr("").0,
                    buffer.as_mut_ptr(),
                    buffer.len() as i32,
                    to_char_ptr("odbcinst.ini").0,
                )
            };
            let driver_path = unsafe { cstr::parse_attribute_string_a(buffer.as_mut_ptr()) };

            let path = Path::new(&driver_path);
            let parent = path.parent().unwrap();
            let log_dir = parent.parent().unwrap().join("log");
            let logfile = Self::file_appender(&log_dir.to_str().unwrap());
            let handle = Self::init_logger(logfile);

            Self { handle, log_dir }
        }) {
            Ok(logger) => Some(logger),
            Err(_) => None,
        }
    }

    pub fn set_level_filter(&self, level_filter: LevelFilter) {
        let config = Config::builder()
            .appender(Appender::builder().build(
                "logfile",
                Box::new(Logger::file_appender(&self.log_dir.to_str().unwrap())),
            ))
            .build(Root::builder().appender("logfile").build(level_filter))
            .unwrap();
        self.handle.set_config(config);
    }

    fn file_appender(log_dir: &str) -> RollingFileAppender {
        let file_path = format!("{log_dir}/mongoodbc.log");
        let roller_pattern = format!("{log_dir}/mongoodbc.log.{{}}");

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
            .build(&file_path, Box::new(policy))
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
        logger
            .as_ref()
            .unwrap()
            .set_level_filter(LevelFilter::Debug);
        info!("info2");
        debug!("debug2");
        error!("error2");
        logger
            .as_ref()
            .unwrap()
            .set_level_filter(LevelFilter::Error);
        info!("info3");
        debug!("debug3");
        error!("error3");
    }
}

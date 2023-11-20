use constants::DRIVER_LOG_VERSION;
use directories::UserDirs;
use lazy_static::lazy_static;
use log::LevelFilter;
use log4rs::{
    append::rolling_file::{
        policy::compound::{
            roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
        },
        RollingFileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    Handle,
};
use shared_sql_utils::odbcinst::DriverSettings;
use std::path::{Path, PathBuf};

const LOG_FILE_SIZE: u64 = 1024 * 500;


// The logger is global to the application.
// The first initialization will create a logger a provide a handle back.
// The logger configuration can then be updated through the handle.
lazy_static! {
    /// Initializes the logger with the given path as its root. The logger will create a logs folder
    /// in the user's documents directory, such as User/Documents/MongoDB/Atlas ODBC Driver/1.1/logs
    ///
    /// If the given path is empty, or there is an error accessing the path, the logger will write its logs to the temp directory.
    ///
    /// The logger is wrapped in a catch_unwind so that logger failure does not cause the driver to crash. In this case
    /// the logger returns None and no logs will be written.
    static ref LOGGER: Option<Logger> = {
        let driver_settings: DriverSettings =
            DriverSettings::from_private_profile_string().unwrap_or_default();

        // Due to numerous reasons why the logger could fail to initialize, we wrap it in a catch_unwind
        // so that logger failure does not cause our dll to crash.
        match std::panic::catch_unwind(|| {
            let log_dir = Logger::get_log_dir(driver_settings.driver);
            if let Some(log_dir_str) = log_dir.to_str() {
                if let Ok(appender) = Logger::file_appender(log_dir_str) {
                    let level_filter = Logger::level_filter_from_string(driver_settings.log_level);
                    match Logger::init_logger(appender, level_filter) {
                        Ok(handle) => Some(Logger { handle, log_dir }),
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
    };
}

#[derive(Debug)]
pub struct Logger {
    handle: Handle,
    log_dir: PathBuf,
}

impl Logger {
    /// Update the logger log level.
    /// This change will affect all logging even already opened connections.
    pub fn set_log_level(level_filter: String) {
        if LOGGER.is_some() {
            let logger = LOGGER.as_ref().unwrap();
            let level_filter = Self::level_filter_from_string(level_filter);
            if let Some(log_dir) = logger.log_dir.to_str() {
                log::logger().flush();
                if let Ok(appender) = Logger::file_appender(log_dir) {
                    let config = Config::builder()
                        .appender(Appender::builder().build("logfile", Box::new(appender)))
                        .build(Root::builder().appender("logfile").build(level_filter))
                        .unwrap();
                    logger.handle.set_config(config);
                    log::logger().flush();
                }
            }
        }
    }

    /// Convert a String value into the corresponding LevelFilter.
    /// If the value does not match a defined filter, it defaults to INFO.
    pub fn level_filter_from_string(level_filter: String) -> LevelFilter {
        match level_filter.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            "off" => LevelFilter::Off,
            _ => LevelFilter::Info,
        }
    }

    /// Create the file appender configuration to pass to the logger.
    fn file_appender(log_dir: &str) -> Result<RollingFileAppender, std::io::Error> {
        let file_path = Path::new(log_dir)
            .join("mongo_odbc.log")
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        let roller_pattern = Path::new(log_dir)
            .join("mongo_odbc.log.{}")
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();

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

    /// Get the logging directory path.
    /// This is useful to check the content of the log files.
    pub fn get_log_dir(driver_path: String) -> PathBuf {
        if driver_path.is_empty() {
            std::env::temp_dir()
        } else if let Some(user_dir) = UserDirs::new() {
            let log_dir = user_dir
                .document_dir()
                .map(|p| {
                    p.join("MongoDB")
                        .join("Atlas SQL ODBC")
                        .join(DRIVER_LOG_VERSION.as_str())
                        .join("logs")
                })
                .unwrap_or_else(std::env::temp_dir);
            if !log_dir.exists() {
                std::fs::create_dir_all(&log_dir).unwrap();
            }
            log_dir
        } else {
            std::env::temp_dir()
        }
    }

    /// Initialize the logger with only one file appender.
    fn init_logger(logfile: RollingFileAppender, loglevel: LevelFilter) -> Result<Handle, ()> {
        let config_res = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(loglevel))
            .map_err(|_e| ());
        if config_res.is_err() {
            return Err(());
        }
        let config = config_res.unwrap();
        let init_res = log4rs::init_config(config);
        if init_res.is_err() {
            return Err(());
        }

        init_res.map_err(|_e| ())
    }
}

#[cfg(test)]
mod driver {

    use std::fs;

    use super::*;
    use log::{debug, error, info};

    #[test]
    fn logger() {
        let log_dir = Logger::get_log_dir("".to_string());
        let tmp_log = log_dir.join("mongo_odbc.log");
        // ensure we remove the log file if it exists. We don't care if it errors since
        // that means it doesn't exist (most likely), or we don't have permissions to touch
        // it, which will make this test very hard to run anyway!
        fs::remove_file(&tmp_log).unwrap_or_default();
        Logger::set_log_level("info".to_string());

        info!("info1");
        debug!("debug1");
        error!("error1");

        let mut log_file = fs::read_to_string(&tmp_log).unwrap();
        assert!(log_file.contains("info1"));
        assert!(log_file.contains("error1"));
        assert!(!log_file.contains("debug1"));

        Logger::set_log_level("debug".to_string());

        info!("info2");
        debug!("debug2");
        error!("error2");

        log_file = fs::read_to_string(&tmp_log).unwrap();

        assert!(log_file.contains("info2"));
        assert!(log_file.contains("error2"));
        assert!(log_file.contains("debug2"));

        Logger::set_log_level("error".to_string());

        info!("info3");
        debug!("debug3");
        error!("error3");

        log_file = fs::read_to_string(&tmp_log).unwrap();

        assert!(log_file.contains("error3"));
        assert!(!log_file.contains("info3"));
        assert!(!log_file.contains("debug3"));

        fs::remove_file(tmp_log).unwrap()
    }
}

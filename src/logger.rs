use env_logger::Builder;
use log::{info, LevelFilter};
use std::str::FromStr;

/// Initializes the logging framework for the application with a specified verbosity level.
///
/// This function configures the global logging behavior based on the provided `log_level`
/// argument. It parses the `log_level` string into a `LevelFilter`, applying it to the logger.
/// If the parsing fails, it defaults to `LevelFilter::Info`, ensuring that logging is always
/// enabled at a reasonable level. This function also prints the initial log message indicating
/// the start of the application and the active logging level.
///
/// # Arguments
///
/// * `log_level` - A string slice that specifies the desired logging level (e.g., "info", "debug").
pub fn init_logging(log_level: &str) {
    let level = LevelFilter::from_str(log_level).unwrap_or_else(|_| {
        eprintln!("Invalid log level: {}. Defaulting to 'info'.", log_level);
        LevelFilter::Info
    });

    Builder::new().filter_level(level).init();

    info!("Application has started with log level: {}", log_level);
}

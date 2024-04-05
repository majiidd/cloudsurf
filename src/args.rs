use clap::Parser;

const DEFAULT_COUNT: u8 = 5;

/// Defines command-line arguments for the application.
///
/// This struct is used by `clap` to parse command-line arguments. It leverages the `derive` macro
/// from `clap` to automatically generate the parsing code based on the struct fields and their annotations.
#[derive(Parser, Debug)]
#[clap(
    version = "1.0",
    about = "Checks the validity of IP addresses using TLS connections."
)]
pub struct Args {
    /// Number of IP addresses to check.
    ///
    /// Specifies how many IP addresses the application should attempt to connect to.
    #[clap(short, long, default_value_t = DEFAULT_COUNT)]
    pub count: u8,

    /// Comma-separated list of IP address prefixes to skip.
    ///
    /// Provides the capability to exclude certain IP prefixes from being checked.
    #[clap(
        long,
        help = "List of comma-separated IP address prefixes to skip. Example: --skip-prefixes \"192.168,10.0,172\""
    )]
    pub skip_prefixes: Option<String>,

    /// File path to read or write IP addresses.
    ///
    /// This argument is now required and specifies the file path where IP addresses
    /// will be written to.
    #[clap(
        short = 'f',
        long,
        help = "Path to the file for  writing IP addresses. This argument is optional."
    )]
    pub file_path: Option<String>,

    /// Logging level to control the verbosity of the application's output.
    ///
    /// Determines the amount of log information the application will output
    #[clap(
        long,
        env = "RUST_LOG",
        default_value = "info",
        help = "Sets the logging level for the application's output. Defaults to 'info' if not specified."
    )]
    pub log_level: String,
}

#[cfg(test)]
mod tests {
    use super::*; 
    use clap::error::ErrorKind;

    #[test]
    fn test_default_values() {
        let args = Args::parse_from(["testapp"]);
        assert_eq!(args.count, 5);
        assert_eq!(args.log_level, "info");
        assert!(args.skip_prefixes.is_none());
        assert!(args.file_path.is_none());
    }

    #[test]
    fn test_valid_input() {
        let args = Args::parse_from([
            "testapp",
            "--count", "10",
            "--skip-prefixes", "192.168,10.0",
            "-f", "/path/to/file",
            "--log-level", "debug"
        ]);

        assert_eq!(args.count, 10);
        assert_eq!(args.skip_prefixes, Some("192.168,10.0".to_string()));
        assert_eq!(args.file_path, Some("/path/to/file".to_string()));
        assert_eq!(args.log_level, "debug");
    }

    #[test]
    fn test_invalid_count() {
        let result = Args::try_parse_from(["testapp", "--count", "not_a_number"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ValueValidation);
    }
}

use anyhow::{Context, Result};
use log::info;
use std::fs::File;
use std::io::{self, Write};
use std::net::Ipv4Addr;

/// A predefined list of operator domains and their corresponding operator names.
const OPERATOR_DOMAINS: [(&str, &str); 27] = [
    ("mci.ircf.space", "MCI"),
    ("mcic.ircf.space", "MCI"),
    ("mtn.ircf.space", "MTN"),
    ("mtnc.ircf.space", "MTN"),
    ("mkh.ircf.space", "MKH"),
    ("rtl.ircf.space", "RTL"),
    ("hwb.ircf.space", "HWB"),
    ("ast.ircf.space", "AST"),
    ("sht.ircf.space", "SHT"),
    ("prs.ircf.space", "PRS"),
    ("mbt.ircf.space", "MBT"),
    ("ask.ircf.space", "ASK"),
    ("rsp.ircf.space", "RSP"),
    ("afn.ircf.space", "AFN"),
    ("ztl.ircf.space", "ZTL"),
    ("psm.ircf.space", "PSM"),
    ("arx.ircf.space", "ARX"),
    ("smt.ircf.space", "SMT"),
    ("shm.ircf.space", "SHM"),
    ("fnv.ircf.space", "FNV"),
    ("dbn.ircf.space", "DBN"),
    ("apt.ircf.space", "APT"),
    ("fnp.ircf.space", "FNP"),
    ("ryn.ircf.space", "RYN"),
    ("sbn.ircf.space", "SBN"),
    ("ptk.ircf.space", "PTK"),
    ("atc.ircf.space", "ATC"),
];

/// Writes IP addresses and predefined operator domains to a file.
///
/// Each IP address from the input list is written to the file multiple times,
/// once for each operator in a predefined list of operators. Following the IP addresses,
/// a list of predefined operator domains is also written to the file.
///
/// # Arguments
///
/// * `ips` - A list of tuples, each containing an `Ipv4Addr` and a latency measurement (`u128`).
///           The latency is currently not used in the function.
/// * `file_path` - The path to the file where the data will be written.
///
/// # Errors
///
/// Returns an error if the file cannot be created or if writing to the file fails at any point.
///
/// # Examples
///
/// ```
/// use std::net::Ipv4Addr;
/// use your_crate::write_ips_to_file;
///
/// let ips = vec![(Ipv4Addr::new(192, 168, 1, 1), 100)];
/// write_ips_to_file(ips, "output.txt").expect("Failed to write IPs to file");
/// ```
pub fn write_ips_to_file(ips: &Vec<(Ipv4Addr, u128)>, file_path: &str) -> Result<()> {
    let operator_list = vec!["MTN", "MCI", "RTL", "ZTL", "SHT"];

    let mut file = match File::create(file_path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            // Handle permission denied error specifically
            return Err(anyhow::Error::new(e).context(format!("Permission denied when attempting to write to '{}'. Please ensure the application has the necessary permissions, or choose a different location.", file_path)));
        }
        Err(e) => {
            return Err(e.into());
        }
    };

    for (ip, _) in ips {
        for operator in &operator_list {
            writeln!(file, "{} {}", ip, operator)
                .with_context(|| format!("Couldn't write IP and operator to file {}", file_path))?;
        }
    }

    for (domain, operator) in OPERATOR_DOMAINS.iter() {
        writeln!(file, "{} {}", domain, operator)
            .with_context(|| format!("Couldn't write new entries to file {}", file_path))?;
    }

    info!("Successfully wrote to file {}", file_path);

    Ok(())
}

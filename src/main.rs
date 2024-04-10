mod args;
mod logger;
mod network;
mod print;
mod file;

use crate::args::Args;
use crate::logger::init_logging;
use crate::network::fetch_and_filter_ipv4_list;
use crate::network::check_tls_availability;
use crate::file::write_ips_to_file;
use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    init_logging(&args.log_level);

    let skip_prefixes_vec = args.skip_prefixes
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    let filtered_ips = fetch_and_filter_ipv4_list(&skip_prefixes_vec).await?;
    let valid_ips = check_tls_availability(&filtered_ips, &args.domain, args.port, args.count, args.max_valid_ips).await?;

    print::ips(&valid_ips);

    if let Some(path) = &args.file_path {
        write_ips_to_file(&valid_ips, path)?;
    }

    Ok(())
}

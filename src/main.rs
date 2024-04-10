mod args;
mod logger;
mod network;

use crate::args::Args;
use crate::logger::init_logging;
use crate::network::fetch_and_filter_ipv4_list;
use crate::network::check_tls_availability;
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

    let a = fetch_and_filter_ipv4_list(&skip_prefixes_vec).await?;

    let last = check_tls_availability(&a, &args.domain, 443, args.count, args.max_valid_ips).await?;

    println!("{:?}", last);

    Ok(())
}

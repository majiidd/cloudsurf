use std::{
    net::Ipv4Addr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use futures::{stream::FuturesUnordered, StreamExt};
use rand::seq::SliceRandom;
use rustls::{pki_types::ServerName, version, ClientConfig, RootCertStore};
use tokio::{net::TcpStream, time::timeout};
use tokio_rustls::TlsConnector;
use webpki_roots::TLS_SERVER_ROOTS;

/// Default timeout for TCP and TLS connections.
const TIMEOUT: Duration = Duration::from_secs(1);

/// Checks the availability of TLS over a list of IP addresses for a specific domain and port.
///
/// This function takes a list of IPv4 addresses, a domain, a port, and a number `n`. It then attempts
/// to establish a TLS connection to each of the IP addresses on the specified port and domain.
/// It measures the time taken to establish each successful connection and returns a list of the
/// fastest `n` connections.
///
/// # Arguments
///
/// * `ips` - A list of IPv4 addresses to check for TLS availability.
/// * `domain` - The domain name to use for the TLS connection.
/// * `port` - The port number to use for the connection.
/// * `n` - The number of successful connections to return, sorted by connection time.
///
/// # Returns
///
/// A Result containing a vector of tuples, each consisting of an IPv4 address and its connection time in milliseconds,
/// sorted by the fastest connection time. The vector is limited to the `n` fastest connections.
pub async fn check_tls_availability(
    ips: &Vec<Ipv4Addr>,
    domain: &str,
    port: u16,
    attempts: usize,
    n: usize,
) -> Result<Vec<(Ipv4Addr, u128)>> {
    if ips.is_empty() {
        return Ok(Vec::new());
    }

    // Randomly select a subset of IP addresses to test.
    let target: Vec<_> = ips
        .choose_multiple(&mut rand::thread_rng(), attempts)
        .cloned()
        .collect();

    // Prepare the TLS client configuration.
    let config = prepare_tls_config()?;
    let connector = TlsConnector::from(config);

    // Attempt TLS connections to the selected IPs.
    let mut valid_ips = create_connection_tasks(target, domain, port, connector).await?;

    // Sort the valid IP addresses by their connection times.
    valid_ips.sort_by_key(|&(_, elapsed)| elapsed);
    let end = valid_ips.len().min(n); // Limit the results to `n` entries.

    Ok(valid_ips[..end].to_vec())
}

/// Creates and executes asynchronous tasks to attempt TLS connections to a list of IP addresses.
///
/// # Arguments
/// * `target` - A list of IP addresses to attempt connection to.
/// * `domain` - The domain name to use for TLS connections.
/// * `port` - The port number to connect to.
/// * `connector` - A `TlsConnector` instance for making TLS connections.
///
/// # Returns
/// A Result containing a vector of tuples, each with an IP address and its connection time in milliseconds.
async fn create_connection_tasks(
    target: Vec<Ipv4Addr>,
    domain: &str,
    port: u16,
    connector: TlsConnector,
) -> Result<Vec<(Ipv4Addr, u128)>> {
    // Convert the domain to a format suitable for TLS handshake.
    let domain_name = ServerName::try_from(domain.to_string())?;

    // Shared list to hold valid IP addresses and their connection times.
    let valid_ips = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Map each IP address to an asynchronous task that attempts to establish a TLS connection.
    let tasks: Vec<_> = target
        .into_iter()
        .map(|ip| {
            let connector_clone = connector.clone();
            let domain_name_clone = domain_name.clone();
            let valid_ips_clone = valid_ips.clone();
            let addr = format!("{}:{}", ip, port);

            tokio::spawn(async move {
                let start = Instant::now();

                // Attempt to connect with a specified timeout.
                let stream = match timeout(TIMEOUT, TcpStream::connect(&addr)).await {
                    Ok(Ok(s)) => s,
                    _ => return,
                };

                // If the TLS handshake succeeds, record the IP and connection time.
                if timeout(TIMEOUT, connector_clone.connect(domain_name_clone, stream))
                    .await
                    .is_ok()
                {
                    let duration = start.elapsed().as_millis();
                    let mut ips = valid_ips_clone.lock().await;
                    ips.push((ip, duration));
                }
            })
        })
        .collect();

    // Wait for all tasks to complete.
    FuturesUnordered::from_iter(tasks)
        .for_each(|_| async {})
        .await;

    // Retrieve the list of valid IP addresses and their connection times.
    let valid_ips = valid_ips.lock().await;
    Ok(valid_ips.clone())
}

/// Prepares the TLS client configuration with root certificates and TLS version.
///
/// # Returns
/// A Result containing an Arc-wrapped `ClientConfig`, which can be used to establish TLS connections.
fn prepare_tls_config() -> Result<Arc<ClientConfig>> {
    // Initialize the root certificate store from webpki_roots.
    let root_store = RootCertStore {
        roots: TLS_SERVER_ROOTS.to_vec(),
    };

    // Set up the client configuration with TLS version 1.3 and the root certificate store.
    let config = ClientConfig::builder_with_protocol_versions(&[&version::TLS13])
        .with_root_certificates(root_store)
        .with_no_client_auth(); // No client authentication is used for simplicity.

    Ok(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_tls_availability() {
        let ips = vec![
            Ipv4Addr::new(104, 16, 132, 229),
            Ipv4Addr::new(198, 18, 0, 10),
        ];
        let domain = "cloudflare.com";
        let port = 443;
        let count = 10;
        let n = 1;

        let result = check_tls_availability(&ips, domain, port, count, n).await;
        assert!(result.is_ok(), "Expected Ok result, but got an Err");

        let valid_ips = result.unwrap();
        assert!(!valid_ips.is_empty(), "Expected at least one valid IP");
    }

    #[tokio::test]
    async fn test_empty_ip_list() {
        let ips = Vec::new(); // Empty list of IPs
        let domain = "example.com";
        let port = 443;
        let count = 10;
        let n = 1;

        let result = check_tls_availability(&ips, domain, count, port, n).await;
        assert!(result.is_ok(), "Expected Ok result with empty input");
        let valid_ips = result.unwrap();
        assert!(
            valid_ips.is_empty(),
            "Expected no valid IPs with empty input list"
        );
    }

    #[tokio::test]
    async fn test_unreachable_domain() {
        let ips = vec![Ipv4Addr::new(1, 8, 8, 1)]; // Example IP
        let domain = "unreachable.unreachableexample.com"; // Unreachable domain
        let port = 443;
        let count = 10;
        let n = 1;

        let result = check_tls_availability(&ips, domain, count, port, n).await;
        assert!(
            result.is_ok(),
            "Expected Ok result even with unreachable domain"
        );
        let valid_ips = result.unwrap();
        assert!(
            valid_ips.is_empty(),
            "Expected no valid IPs with an unreachable domain"
        );
    }
}

use std::net::Ipv4Addr;

use anyhow::{Context, Result};
use ipnetwork::Ipv4Network;
use reqwest::header;
use serde::Deserialize;

const CLOUDFLARE_API_URL: &str = "https://api.cloudflare.com/client/v4/ips";

/// Represents the top-level response from Cloudflare's IP list API.
///
/// This structure encapsulates the overall response from the Cloudflare API,
/// including the success status, detailed results, and any errors or messages
/// that might be included in the response.
#[derive(Deserialize, Debug)]
struct CloudflareIps {
    /// Indicates whether the API request was successful.
    success: bool,
    /// Contains the detailed result of the API request, including
    /// lists of IPv4 and IPv6 CIDR blocks and an etag for caching purposes.
    result: CloudflareIpResult,
    /// A list of error messages, if any were returned by the API.
    /// This field is typically empty if `success` is `true`.
    errors: Vec<String>,
}

/// Represents the detailed result of a successful request to Cloudflare's IP list API.
///
/// Includes lists of CIDR blocks for both IPv4 and IPv6 addresses that Cloudflare uses,
/// as well as an etag that can be used for caching and conditional requests.
#[derive(Deserialize, Debug)]
struct CloudflareIpResult {
    /// A list of CIDR blocks representing the IPv4 addresses used by Cloudflare.
    ipv4_cidrs: Vec<String>,
}

/// Fetches the list of IPv4 CIDRs from Cloudflare's API, excluding any that start with specified prefixes.
///
/// # Returns
///
/// A `Result` wrapping a vector of filtered IPv4 CIDR strings on success, or an `anyhow::Error` on failure.
async fn fetch_ipv4_cidr_list(url: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(header::ACCEPT_ENCODING, "application/json")
        .send()
        .await
        .context("Failed to send request to Cloudflare API")?
        .json::<CloudflareIps>()
        .await
        .context("Failed to deserialize Cloudflare API response")?;

    if !res.success {
        let error_message = res.errors.join(", ");
        anyhow::bail!(
            "Error fetching CIDR list from Cloudflare: {}",
            error_message
        );
    }

    Ok(res.result.ipv4_cidrs)
}

/// Attempts to expand a list of CIDR blocks into individual IP addresses.
///
/// This function parses each CIDR string and generates the corresponding range of IP addresses.
/// It will return an error if any CIDR block is invalid.
///
/// # Arguments
///
/// * `cidrs` - A slice of strings representing the CIDR blocks to be expanded.
///
/// # Returns
///
/// A `Result` wrapping a vector of `Ipv4Addr` representing individual IP addresses within the CIDR blocks,
/// or an `anyhow::Error` if any CIDR block is invalid.
fn expand_cidrs_to_ips(cidrs: &[String]) -> Result<Vec<Ipv4Addr>> {
    cidrs
        .iter()
        .flat_map(|cidr| {
            let network = match cidr.parse::<Ipv4Network>() {
                Ok(network) => network,
                Err(e) => {
                    return vec![Err(anyhow::anyhow!("Invalid CIDR '{}': {}", cidr, e))].into_iter()
                }
            };
            network.iter().map(Ok).collect::<Vec<_>>().into_iter()
        })
        .collect::<Result<Vec<_>, _>>()
}

/// Filters out IP addresses that start with any of the given prefixes.
/// If `skip_prefixes` is empty, all IPs are included without filtering.
///
/// # Arguments
///
/// * `ips` - A vector of `Ipv4Addr` representing the IP addresses to filter.
/// * `skip_prefixes` - A slice of strings representing the prefixes to filter by.
///
/// # Returns
///
/// A vector of `Ipv4Addr` that do not start with any of the given prefixes, or all IPs if no prefixes are provided.
fn filter_ips_by_prefix(ips: Vec<Ipv4Addr>, skip_prefixes: &[String]) -> Vec<Ipv4Addr> {
    // If skip_prefixes is empty, return all IPs without filtering
    if skip_prefixes.is_empty() {
        return ips;
    }

    ips.into_iter()
        .filter(|ip| {
            !skip_prefixes
                .iter()
                .any(|prefix| ip.to_string().starts_with(prefix))
        })
        .collect()
}

/// Fetches the list of IPv4 addresses used by Cloudflare, expands them from CIDR notation,
/// and filters out any addresses that start with the specified prefixes.
///
/// The purpose of this function is to provide a filtered list of IPv4 addresses based on
/// Cloudflare's publicly used IP ranges, potentially excluding specific subnets as required.
///
/// # Arguments
///
/// * `skip_prefixes` - A vector of string slices (`&str`) representing the prefixes to be excluded
///   from the final list of IP addresses. Each prefix is matched at the start of the IP address strings.
///   If this vector is empty, no filtering is applied, and all IP addresses are returned.
///
/// # Returns
///
/// A `Result<Vec<Ipv4Addr>, anyhow::Error>` which is:
/// - Ok(`Vec<Ipv4Addr>`): A vector of `Ipv4Addr` representing the filtered IPv4 addresses.
/// - Err(`anyhow::Error`): An error encountered during any step of the process, including issues with
///   fetching data from Cloudflare's API, deserializing the response, parsing the CIDR blocks,
///   or handling invalid prefixes in `skip_prefixes`.
///
/// # Examples
///
/// ```no_run
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let skip_prefixes = vec!["192.0.2".to_string(), "198.51.100".to_string()];
///     let filtered_ips = fetch_and_filter_ipv4_list(&skip_prefixes).await?;
///     println!("{:?}", filtered_ips);
///     Ok(())
/// }
/// ```
///
/// This function makes asynchronous network requests and thus must be awaited. Ensure it is called
/// within an async context.
pub async fn fetch_and_filter_ipv4_list(skip_prefixes: &[String]) -> Result<Vec<Ipv4Addr>> {
    let cidr_list = fetch_ipv4_cidr_list(CLOUDFLARE_API_URL).await?;
    let all_ips = expand_cidrs_to_ips(&cidr_list)?;
    let filtered_ips = filter_ips_by_prefix(all_ips, skip_prefixes);

    Ok(filtered_ips)
}

#[cfg(test)]
mod tests {
    use super::*;

    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn test_fetch_ipv4_cidr_list() -> Result<()> {
        let mock_server = MockServer::start().await;
        let response = ResponseTemplate::new(200)
            .insert_header("Content-Type", "application/json")
            .set_body_json(serde_json::json!({
                "success": true,
                "result": {
                    "ipv4_cidrs": [
                        "173.245.48.0/20",
                        "103.21.244.0/22",
                        "103.22.200.0/22",
                        "103.31.4.0/22",
                        "141.101.64.0/18",
                        "108.162.192.0/18",
                        "190.93.240.0/20",
                        "188.114.96.0/20",
                        "197.234.240.0/22",
                        "198.41.128.0/17",
                        "162.158.0.0/15",
                        "104.16.0.0/13",
                        "104.24.0.0/14",
                        "172.64.0.0/13",
                        "131.0.72.0/22"
                    ],
                    "ipv6_cidrs": [
                        "2400:cb00::/32",
                        "2606:4700::/32",
                        "2803:f800::/32",
                        "2405:b500::/32",
                        "2405:8100::/32",
                        "2a06:98c0::/29",
                        "2c0f:f248::/32"
                    ],
                    "etag": "38f79d050aa027e3be3865e495dcc9bc"
                },
                "errors": [],
                "messages": []
            }));

        Mock::given(method("GET"))
            .and(path("/client/v4/ips"))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let url = format!("{}/client/v4/ips", mock_server.uri());
        let result = fetch_ipv4_cidr_list(&url).await;

        assert!(result.is_ok(), "Error: {:?}", result.err());

        let cidrs = result?;
        assert_eq!(cidrs.len(), 15);

        Ok(())
    }

    #[test]
    fn test_expand_cidrs_to_ips() {
        let cidrs = vec!["173.245.48.0/20".to_string(), "104.24.0.0/14".to_string()];
        let expanded = expand_cidrs_to_ips(&cidrs).unwrap();

        assert_eq!(expanded.len(), 262144 + 4096);
    }

    #[test]
    fn test_filter_ips_by_prefix() {
        let ips = vec![
            "192.0.2.1".parse().unwrap(),
            "198.51.100.1".parse().unwrap(),
            "203.0.113.1".parse().unwrap(),
        ];
        let skip_prefixes = vec!["198.51".to_string(), "203".to_string()];
        let filtered = filter_ips_by_prefix(ips, &skip_prefixes);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "192.0.2.1".parse::<Ipv4Addr>().unwrap());
    }
}

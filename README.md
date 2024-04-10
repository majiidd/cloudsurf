# CloudSurf

CloudSurf is a Rust application designed to interact with network services, specifically focusing on Cloudflare integration and TLS certificate checking!

## Getting Started

These instructions will guide you through setting up CloudSurf on your local machine.

### Prerequisites

- Rust and Cargo installed on your system.
- Access to the internet for fetching data and interacting with APIs.

### Installation

1. Clone the repository:

```bash
git clone https://github.com/majiidd/cloudsurf.git
cd cloudsurf
```

2. Build the project:

```bash
cargo build
```

3. Run the application:

```bash
cargo run -- [OPTIONS]
```

Replace `[OPTIONS]` with the command-line arguments you wish to use.

## Usage

To use CloudSurf, execute the built binary with the desired options. Here's an example command:

```bash
cargo run -- --domain example.com --port 443
```

For a full list of options, use the `--help` flag:

```bash
cargo run -- --help
```

## Running Tests

To ensure CloudSurf is functioning correctly, run the included test suite:

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

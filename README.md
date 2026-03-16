# onyphe-rs

A fast, concurrent, and robust command-line interface for the [ONYPHE API](https://www.onyphe.io/), written in Rust.

**`onyphe-rs`** is a direct vibe coded Rust port of the official [onyphe/cli](https://github.com/onyphe/cli) tool, offering better performance and deeper integration with Unix pipelines.

## Features

- **Blazing Fast**: Leverages Rust's async ecosystem (Tokio) for high performance.
- **Full API Coverage**: Supports User, Search, Summary, Simple, Simple Best, and Export APIs.
- **Pipeline Integration**: Designed to be chained seamlessly with other CLI tools using the `--aslines` feature.
- **Flexible Configuration**: Uses `~/.onyphe.ini` or environment variables for API key management.

## Installation

Ensure you have [Rust and Cargo](https://rustup.rs/) installed, then run:

```bash
cargo build --release
# The binary will be available at target/release/onyphe-rs
```

## Quick Start

### Configuration

Create a configuration file at `~/.onyphe.ini`:

```ini
[main]
api_key=YOUR_API_KEY
```

Alternatively, you can provide the key directly using `--key YOUR_API_KEY`.

### Usage Examples

#### 1. Check your license information

```bash
onyphe-rs user
```

#### 2. Search API

Search for all Apache web servers on standard HTTP ports:

```bash
onyphe-rs search "product:Apache port:80"
```

#### 3. Summary API

Get a summary for an IP address:

```bash
onyphe-rs summary ip "1.1.1.1"
```

#### 4. Pipeline Support

You can output the results as raw JSON lines using `--aslines`, which is perfect for processing with `jq`:

```bash
onyphe-rs search "category:datascan" --aslines | jq -r '.ip'
```

## Usage Reference

```text
ONYPHE Command Line Interface

Usage: onyphe-rs [OPTIONS] <COMMAND>

Commands:
  user         Get information on your license
  search       Use Search API for query
  summary      Use Summary API for query
  simple       Use Simple API for query
  simple-best  Use Simple Best API for query
  export       Export API for query
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose            Verbosity level
      --config <CONFIG>    Config file path [default: ~/.onyphe.ini]
      --key <KEY>          API Key
      --maxpage <MAXPAGE>  Max page to fetch with Search API
      --size <SIZE>        Fetch pages of size results
      --aslines            Output scan results as lines of JSON
  -h, --help               Print help
  -V, --version            Print version
```

## Acknowledgements

This tool is a Rust port of the original [onyphe/cli](https://github.com/onyphe/cli), aiming to provide the same functionality with the speed and safety of Rust.

//! Ad-hoc live-fetch tool: logs into Schwab once (a real browser-based
//! OAuth flow, via `nest-auth-oauth-client`'s HTTPS loopback callback),
//! then fetches whatever endpoints are listed in a manifest file and
//! writes each raw response to its own JSON file.
//!
//! Not part of this crate's public API — a development tool for capturing
//! real response shapes to build typed structs against, since Schwab's own
//! API reference is gated behind a login this environment doesn't have.
//! Credentials are read from the environment, never passed on the command
//! line or committed anywhere.
//!
//! ```text
//! SCHWAB_APP_KEY=... SCHWAB_APP_SECRET=... \
//!   cargo run -p nest-schwab --example fetch_shapes -- <manifest.json> <output_dir>
//! ```
//!
//! Manifest format (JSON array):
//!
//! ```json
//! [
//!   {"label": "pricehistory_aapl", "kind": "market_data", "path": "/pricehistory?symbol=AAPL&periodType=year&period=1&frequencyType=daily"},
//!   {"label": "accounts", "kind": "trader", "path": "/accounts"}
//! ]
//! ```
//!
//! `kind` is `"market_data"` or `"trader"` — whichever base URL the path
//! is relative to. `path` may already include a query string.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use nest_auth_oauth_client::{OAuthClient, OAuthTokenAuth};
use nest_schwab::{SchwabClient, SchwabConfig};
use serde::Deserialize;

#[derive(Deserialize)]
struct FetchSpec {
    label: String,
    kind: String,
    path: String,
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let manifest_path = args
        .next()
        .expect("usage: fetch_shapes <manifest.json> <output_dir>");
    let output_dir = args
        .next()
        .expect("usage: fetch_shapes <manifest.json> <output_dir>");

    let app_key = std::env::var("SCHWAB_APP_KEY").expect("SCHWAB_APP_KEY not set");
    let app_secret = std::env::var("SCHWAB_APP_SECRET").expect("SCHWAB_APP_SECRET not set");

    let manifest_json = fs::read_to_string(&manifest_path).expect("failed to read manifest");
    let specs: Vec<FetchSpec> =
        serde_json::from_str(&manifest_json).expect("invalid manifest JSON");

    let config = SchwabConfig::new(app_key, app_secret);
    let oauth_client =
        OAuthClient::new(&config.to_oauth_client_config()).expect("build oauth client");

    let request = oauth_client.authorization_request();
    println!("Open this URL in a browser and log in:\n{}", request.url);
    println!("(Your browser will warn about the self-signed certificate on the 127.0.0.1 redirect — that's expected, click through it.)");
    println!("Waiting up to 5 minutes for the redirect...");

    let token = oauth_client
        .complete_login(request, Duration::from_secs(300))
        .await
        .expect("login failed");
    println!(
        "Login succeeded (access token acquired, expires_at_ms={:?}).",
        token.expires_at_ms
    );

    let auth = OAuthTokenAuth::new(token);
    let client = SchwabClient::new(&config, auth).expect("build schwab client");

    fs::create_dir_all(&output_dir).expect("failed to create output dir");

    for spec in specs {
        println!("Fetching {} ({})...", spec.label, spec.path);
        let result = match spec.kind.as_str() {
            "market_data" => client.get_market_data(&spec.path, &[]).await,
            "trader" => client.get_trader(&spec.path).await,
            other => {
                eprintln!("  unknown kind '{other}' for {}, skipping", spec.label);
                continue;
            }
        };

        match result {
            Ok(value) => {
                let pretty = serde_json::to_string_pretty(&value).expect("serialize response");
                let out_path = PathBuf::from(&output_dir).join(format!("{}.json", spec.label));
                fs::write(&out_path, pretty).expect("failed to write output file");
                println!("  wrote {}", out_path.display());
            }
            Err(error) => {
                eprintln!("  failed: {error}");
            }
        }
    }

    println!("Done.");
}

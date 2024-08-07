//! Configuration for kv-vault capability provider
//!

use core::time::Duration;

use std::collections::HashMap;
use std::env;

use anyhow::{Context, Result};
use tracing::warn;
use url::Url;
use wasmcloud_provider_sdk::{core::secrets::SecretValue, LinkConfig};

use crate::TOKEN_REFRESH_INTERVAL;

/// Default address at which Vault is expected to be running,
/// used if unspecified by configuration
const DEFAULT_VAULT_ADDR: &str = "http://127.0.0.1:8200";

/// KV-Vault configuration
#[derive(Clone, Debug)]
pub struct Config {
    /// Token for connecting to vault, can be set in environment with VAULT_TOKEN.
    /// Required
    pub token: String,
    /// Url for connecting to vault, can be set in environment with VAULT_ADDR.
    /// Defaults to 'http://127.0.0.1:8200'
    pub addr: Url,
    /// Vault mount point, can be set with in environment with VAULT_MOUNT.
    /// Defaults to "secret/"
    pub mount: String,
    /// certificate files - path to CA certificate file(s). Setting this enables TLS
    /// The linkdef value `certs` and the environment variable `VAULT_CERTS`
    /// are parsed as a comma-separated string of file paths to generate this list.
    pub certs: Vec<String>,

    /// Renewal TTL for tokens used by this provider. Defaults to 72 hours.
    pub token_increment_ttl: Option<String>,

    /// Refresh interval for tokens used by this provider. Defaults to 12 hours.
    pub token_refresh_interval: Option<std::time::Duration>,
}

impl Default for Config {
    /// default constructor - Gets all values from environment & defaults
    fn default() -> Self {
        Self::from_values(&HashMap::new()).unwrap()
    }
}

impl Config {
    /// Initialize from a [`LinkConfig`]
    pub fn from_link_config(link_config: &LinkConfig) -> Result<Config> {
        let mut map = HashMap::clone(link_config.config);

        // Attempt to retrieve the vault token from secrets
        if let Some(token) = env::var("VAULT_TOKEN").ok().or_else(|| {
            link_config
                .secrets
                .get("token")
                .and_then(SecretValue::as_string)
                .map(String::from)
        }) {
            map.insert("VAULT_TOKEN".into(), token);
        } else {
            warn!("Secret value [token] (ENV: VAULT_TOKEN) was not found in env or secrets. Please prefer ENV variables or secrets for sensitive values.")
        }

        Self::from_values(&map)
    }

    /// Initialize from linkdef values, environment, and defaults
    ///
    /// NOTE: Prefer [`Self::from_link_config`] rather than this method directly
    pub fn from_values(values: &HashMap<String, String>) -> Result<Config> {
        let addr = env::var("VAULT_ADDR")
            .ok()
            .or_else(|| values.get("addr").cloned())
            .or_else(|| values.get("ADDR").cloned())
            .unwrap_or_else(|| DEFAULT_VAULT_ADDR.to_string());
        let addr = addr.parse().unwrap_or_else(|_| {
            eprintln!(
                "Could not parse VAULT_ADDR [{addr}] as Url, using default of {DEFAULT_VAULT_ADDR}"
            );
            DEFAULT_VAULT_ADDR.parse().unwrap()
        });
        let token = env::var("VAULT_TOKEN")
            .ok()
            .or_else(|| values.get("token").cloned())
            .or_else(|| values.get("TOKEN").cloned())
            .context("missing setting for 'token' or VAULT_TOKEN")?;
        let mount = env::var("VAULT_MOUNT")
            .ok()
            .or_else(|| values.get("mount").cloned())
            .or_else(|| values.get("MOUNT").cloned())
            .unwrap_or_else(|| "secret".to_string());
        let certs = env::var("VAULT_CERTS")
            .ok()
            .or_else(|| values.get("certs").cloned())
            .or_else(|| values.get("CERTS").cloned())
            .map(|certs| certs.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        Ok(Config {
            addr,
            token,
            mount,
            certs,
            token_increment_ttl: env::var("VAULT_TOKEN_INCREMENT_TTL")
                .ok()
                .or_else(|| values.get("token_increment_ttl").cloned())
                .or_else(|| values.get("TOKEN_INCREMENT_TTL").cloned()),
            token_refresh_interval: match env::var("VAULT_TOKEN_REFRESH_INTERVAL")
                .ok()
                .or_else(|| values.get("token_refresh_interval").cloned())
                .or_else(|| values.get("TOKEN_REFRESH_INTERVAL").cloned())
            {
                Some(val) => {
                    let secs = val.parse::<u64>().unwrap_or_else(|_| {
                        eprintln!(
                            "Could not parse VAULT_TOKEN_REFRESH_INTERVAL as u64, using default of {}",
                            TOKEN_REFRESH_INTERVAL.as_secs()
                        );
                        TOKEN_REFRESH_INTERVAL.as_secs()
                    });
                    Some(Duration::from_secs(secs))
                }
                _ => None,
            },
        })
    }
}

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
pub struct AuthOpts {
    /// OCI username, if omitted anonymous authentication will be used
    #[clap(
        short = 'u',
        long = "user",
        env = "WASH_REG_USER",
        hide_env_values = true
    )]
    pub user: Option<String>,

    /// OCI password, if omitted anonymous authentication will be used
    #[clap(
        short = 'p',
        long = "password",
        env = "WASH_REG_PASSWORD",
        hide_env_values = true
    )]
    pub password: Option<String>,

    /// Allow insecure (HTTP) registry connections
    #[clap(long = "insecure")]
    pub insecure: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum RegistryCommand {
    /// Pull an artifact from an OCI compliant registry
    #[clap(name = "pull")]
    Pull(RegistryPullCommand),
    /// Push an artifact to an OCI compliant registry
    #[clap(name = "push")]
    Push(RegistryPushCommand),
    /// Ping (test url) to see if the OCI url has an artifact
    #[clap(name = "ping")]
    Ping(RegistryPingCommand),
}

#[derive(Parser, Debug, Clone)]
pub struct RegistryPullCommand {
    /// URL of artifact
    #[clap(name = "url")]
    pub url: String,

    /// File destination of artifact
    #[clap(long = "destination")]
    pub destination: Option<String>,

    /// Registry of artifact. This is only needed if the URL is not a full (OCI) artifact URL (ie, missing the registry fragment)
    #[clap(short = 'r', long = "registry", env = "WASH_REG_URL")]
    pub registry: Option<String>,

    /// Digest to verify artifact against
    #[clap(short = 'd', long = "digest")]
    pub digest: Option<String>,

    /// Allow latest artifact tags
    #[clap(long = "allow-latest")]
    pub allow_latest: bool,

    #[clap(flatten)]
    pub opts: AuthOpts,
}

#[derive(Parser, Debug, Clone)]
pub struct RegistryPushCommand {
    /// URL to push artifact to
    #[clap(name = "url")]
    pub url: String,

    /// Path to artifact to push
    #[clap(name = "artifact")]
    pub artifact: String,

    /// Registry of artifact. This is only needed if the URL is not a full (OCI) artifact URL (ie, missing the registry fragment)
    #[clap(short = 'r', long = "registry", env = "WASH_REG_URL")]
    pub registry: Option<String>,

    /// Path to config file, if omitted will default to a blank configuration
    #[clap(short = 'c', long = "config")]
    pub config: Option<PathBuf>,

    /// Allow latest artifact tags
    #[clap(long = "allow-latest")]
    pub allow_latest: bool,

    /// Optional set of annotations to apply to the OCI artifact manifest
    #[clap(short = 'a', long = "annotation", name = "annotations")]
    pub annotations: Option<Vec<String>>,

    #[clap(flatten)]
    pub opts: AuthOpts,
}

#[derive(Parser, Debug, Clone)]
pub struct RegistryPingCommand {
    /// URL of artifact
    #[clap(name = "url")]
    pub url: String,

    /// Registry of artifact. This is only needed if the URL is not a full (OCI) artifact URL (ie, missing the registry fragment)
    #[clap(short = 'r', long = "registry", env = "WASH_REG_URL")]
    pub registry: Option<String>,

    #[clap(flatten)]
    pub opts: AuthOpts,
}

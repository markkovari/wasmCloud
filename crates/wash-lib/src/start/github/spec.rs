use serde::Deserialize;

/// GitHubRelease represents the necessary fields to determine wadm and/or wasmCloud
/// has new patch version available. The fields are based on the response from the
/// GitHub release (https://developer.github.com/v3/repos/releases/).
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub published_at: String,
}

impl GitHubRelease {
    pub fn new(tag_name: String, name: String, published_at: String) -> Self {
        Self {
            tag_name,
            name,
            published_at,
        }
    }
}

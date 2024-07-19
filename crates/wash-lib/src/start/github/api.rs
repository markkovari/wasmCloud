use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

type DateTimeUtc = DateTime<Utc>;

const GITHUB_PER_PAGE: u32 = 20000;

async fn get_sorted_releases_of(
    owner: String,
    repo: String,
) -> Result<Vec<GitHubRelease>, reqwest::Error> {
    let wasm_cloud_releases = fetch_latest_releases(owner, repo).await?;

    let mut wasm_cloud_releases = wasm_cloud_releases
        .into_iter()
        .filter(GitHubRelease::is_not_draft_or_pre_release)
        .collect::<Vec<GitHubRelease>>();
    wasm_cloud_releases.sort_by(|a, b| a.published_at.cmp(&b.published_at));
    Ok(wasm_cloud_releases)
}

/// GitHubRelease represents the necessary fields to determine wadm and/or wasmCloud
/// has new patch version available. The fields are based on the response from the
/// GitHub release (https://developer.github.com/v3/repos/releases/).
///
#[derive(Deserialize, Serialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    #[serde(with = "github_date_format")]
    pub published_at: DateTimeUtc,
    pub draft: bool,
    pub prerelease: bool,
}

impl GitHubRelease {
    pub fn is_not_draft_or_pre_release(&self) -> bool {
        !self.draft && !self.prerelease
    }
    pub fn is_main_release(&self) -> Option<semver::Version> {
        let tag_name = self.tag_name.strip_prefix("v").unwrap_or(&self.tag_name);
        // TODO: should be a simple call to map to option
        match semver::Version::parse(tag_name) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

/// Returns the URL to fetch the latest release from the GitHub repository.
/// doc: https://developer.github.com/v3/repos/releases/#get-the-latest-release
fn format_latest_releases(owner: String, repo: String) -> String {
    format!(
        "https://api.github.com/repos/{}/{}/releases/latest?page=0&page={}",
        owner, repo, GITHUB_PER_PAGE
    )
}

async fn fetch_latest_releases(
    owner: String,
    repo: String,
) -> Result<Vec<GitHubRelease>, reqwest::Error> {
    let url = format_latest_releases(owner, repo);
    let response = reqwest::get(&url).await?;
    let release = response.json::<Vec<GitHubRelease>>().await?;
    Ok(release)
}

// TODO: find any chrono serde implementation that can be used instead of this.
mod github_date_format {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%SZ";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    /// Test if the GitHubRelease struct is parsed correctly from the raw string.
    /// Removed some items from the raw string to keep the test readable.
    #[test]
    fn test_github_release_is_parsed_correctly() {
        let raw_string = r#####"
        {
            "url": "https://api.github.com/repos/wasmCloud/wasmCloud/releases/165886656",
            "assets_url": "https://api.github.com/repos/wasmCloud/wasmCloud/releases/165886656/assets",
            "upload_url": "https://uploads.github.com/repos/wasmCloud/wasmCloud/releases/165886656/assets{?name,label}",
            "html_url": "https://github.com/wasmCloud/wasmCloud/releases/tag/washboard-ui-v0.4.0",
            "id": 165886656,
            "node_id": "RE_kwDOEiTU7M4J4zrA",
            "tag_name": "washboard-ui-v0.4.0",
            "target_commitish": "main",
            "name": "washboard-ui-v0.4.0",
            "draft": false,
            "prerelease": false,
            "created_at": "2024-07-17T14:47:54Z",
            "published_at": "2024-07-17T16:15:15Z",
            "tarball_url": "https://api.github.com/repos/wasmCloud/wasmCloud/tarball/washboard-ui-v0.4.0",
            "zipball_url": "https://api.github.com/repos/wasmCloud/wasmCloud/zipball/washboard-ui-v0.4.0",
            "mentions_count": 5
        }
        "#####;

        let release = serde_json::from_str::<GitHubRelease>(raw_string);
        assert!(release.is_ok());
        let release = release.unwrap();
        assert_eq!(release.tag_name, "washboard-ui-v0.4.0");
        assert_eq!(release.name, "washboard-ui-v0.4.0");

        let exptexted_date = NaiveDate::from_ymd_opt(2024, 07, 17)
            .unwrap()
            .and_hms_opt(16, 15, 15)
            .unwrap()
            .and_utc();
        assert_eq!(release.published_at, exptexted_date);
        assert_eq!(release.draft, false);
        assert_eq!(release.prerelease, false);
    }

    #[test]
    fn test_github_release_is_not_draft_or_pre_release() {
        let release = GitHubRelease {
            tag_name: "v0.4.0".to_string(),
            name: "v0.4.0".to_string(),
            published_at: Utc::now(),
            draft: false,
            prerelease: false,
        };
        assert!(release.is_not_draft_or_pre_release());
    }

    #[test]
    fn test_semver_without_prefix() {
        let release = GitHubRelease {
            tag_name: "v0.4.0".to_string(),
            name: "v0.4.0".to_string(),
            published_at: Utc::now(),
            draft: false,
            prerelease: false,
        };
        let version = release.is_main_release();
        assert!(version.is_some());
        assert_eq!(version.unwrap(), semver::Version::parse("0.4.0").unwrap());

        let release_with_prefix = GitHubRelease {
            tag_name: "washboard-ui-v0.4.0".to_string(),
            name: "washboard-ui-v0.4.0".to_string(),
            published_at: Utc::now(),
            draft: false,
            prerelease: false,
        };
        let version = release_with_prefix.is_main_release();
        assert!(version.is_none());
    }
}

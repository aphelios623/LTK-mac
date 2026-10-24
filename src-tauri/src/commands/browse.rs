use crate::deep_link;
use crate::error::{AppError, AppResult, IpcResult, MutexResultExt};
use crate::mods::{InstalledMod, ModLibraryState};
use crate::patcher::PatcherState;
use crate::state::SettingsState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, State};
use ts_rs::TS;
use url::Url;

use super::mods::reject_if_patcher_running;

const REPO_TREE_URL: &str =
    "https://api.github.com/repos/bettie9/LeagueSkins/git/trees/main?recursive=1";
const REPO_RAW_BASE: &str = "https://raw.githubusercontent.com/bettie9/LeagueSkins/main/";
const CACHE_TTL: Duration = Duration::from_secs(60 * 60);
const MAX_PATH_LEN: usize = 256;

static LISTING_CACHE: Mutex<Option<(Instant, Vec<BrowseSkin>)>> = Mutex::new(None);

/// One base skin available in the community mirror.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BrowseSkin {
    pub champion: String,
    pub name: String,
    pub path: String,
}

#[derive(Deserialize)]
struct TreeResponse {
    tree: Vec<TreeEntry>,
    #[serde(default)]
    truncated: bool,
}

#[derive(Deserialize)]
struct TreeEntry {
    path: String,
    #[serde(rename = "type")]
    kind: String,
}

fn fetch_listing() -> AppResult<Vec<BrowseSkin>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("ltk-manager-macos-browse")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Other(format!("Could not create HTTP client: {e}")))?;

    let response = client
        .get(REPO_TREE_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| AppError::Other(format!("Could not reach GitHub: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::Other(format!(
            "GitHub returned status {} while listing skins (rate limit?)",
            response.status()
        )));
    }

    let body = response
        .text()
        .map_err(|e| AppError::Other(format!("Could not read GitHub response: {e}")))?;
    let parsed: TreeResponse = serde_json::from_str(&body)
        .map_err(|e| AppError::Other(format!("Unexpected GitHub response: {e}")))?;
    if parsed.truncated {
        tracing::warn!("GitHub truncated the skin file list; some skins may be missing");
    }

    // Base skins only: skins/<Champion>/<Skin Name>.fantome
    let mut skins: Vec<BrowseSkin> = parsed
        .tree
        .into_iter()
        .filter(|e| e.kind == "blob")
        .filter_map(|e| {
            let parts: Vec<&str> = e.path.split('/').collect();
            if parts.len() != 3 || parts[0] != "skins" {
                return None;
            }
            let name = parts[2].strip_suffix(".fantome")?;
            Some(BrowseSkin {
                champion: parts[1].to_string(),
                name: name.to_string(),
                path: e.path.clone(),
            })
        })
        .collect();
    skins.sort_by(|a, b| a.champion.cmp(&b.champion).then(a.name.cmp(&b.name)));
    Ok(skins)
}

fn list_skins(force: bool) -> AppResult<Vec<BrowseSkin>> {
    if !force {
        if let Ok(guard) = LISTING_CACHE.lock() {
            if let Some((at, skins)) = guard.as_ref() {
                if at.elapsed() < CACHE_TTL {
                    return Ok(skins.clone());
                }
            }
        }
    }
    let skins = fetch_listing()?;
    if let Ok(mut guard) = LISTING_CACHE.lock() {
        *guard = Some((Instant::now(), skins.clone()));
    }
    Ok(skins)
}

/// Build the download URL for a skin path. Only `skins/<Champion>/<Name>.fantome`
/// is accepted, and the host and repo are fixed.
fn skin_url(path: &str) -> AppResult<String> {
    let bad = || AppError::ValidationFailed("Not a valid skin path".into());
    if path.is_empty()
        || path.len() > MAX_PATH_LEN
        || path.chars().any(|c| c.is_control() || c == '\\')
    {
        return Err(bad());
    }
    let segments: Vec<&str> = path.split('/').collect();
    let ok_shape = segments.len() == 3
        && segments[0] == "skins"
        && segments[2].ends_with(".fantome")
        && segments
            .iter()
            .all(|s| !s.is_empty() && *s != "." && *s != "..");
    if !ok_shape {
        return Err(bad());
    }

    let mut url =
        Url::parse(REPO_RAW_BASE).map_err(|e| AppError::Other(format!("Bad base URL: {e}")))?;
    url.path_segments_mut()
        .map_err(|_| AppError::Other("Base URL cannot be extended".into()))?
        .pop_if_empty()
        .extend(segments);
    Ok(url.to_string())
}

/// List base skins from the community mirror (cached for an hour unless `force`).
#[tauri::command]
pub fn browse_list_skins(force: Option<bool>) -> IpcResult<Vec<BrowseSkin>> {
    list_skins(force.unwrap_or(false)).into()
}

/// Download one skin from the mirror and install it into the library.
#[tauri::command]
pub fn browse_install_skin(
    path: String,
    app_handle: AppHandle,
    library: State<ModLibraryState>,
    settings: State<SettingsState>,
    patcher: State<PatcherState>,
) -> IpcResult<InstalledMod> {
    let result: AppResult<InstalledMod> = (|| {
        reject_if_patcher_running(&patcher)?;
        let url = skin_url(&path)?;
        tracing::info!("Browse install: {}", url);

        let temp_path = deep_link::download_mod_file(&url, &app_handle)?;
        let temp_path_str = temp_path.to_string_lossy().to_string();

        let current = settings.0.lock().mutex_err()?.clone();
        let installed = library.0.install_mod_from_package(&current, &temp_path_str);

        if let Err(e) = std::fs::remove_file(&temp_path) {
            tracing::warn!("Failed to clean up temp file: {}", e);
        }
        deep_link::emit_install_complete(&app_handle);

        installed
    })();
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_encoded_url() {
        let u = skin_url("skins/Akali/Blood Moon Akali.fantome").unwrap();
        assert_eq!(
            u,
            "https://raw.githubusercontent.com/bettie9/LeagueSkins/main/skins/Akali/Blood%20Moon%20Akali.fantome"
        );
    }

    #[test]
    fn rejects_bad_paths() {
        for p in [
            "",
            "skins/../x.fantome",
            "skins/Akali/x.zip",
            "emotes/Akali/x.fantome",
            "skins/Akali/Coven Akali/Ruby.fantome",
            "/skins/Akali/x.fantome",
        ] {
            assert!(skin_url(p).is_err(), "{p}");
        }
    }
}

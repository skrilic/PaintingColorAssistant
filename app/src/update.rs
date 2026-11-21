//! "Is there a newer version?": asks GitHub for the latest release, only when the user
//! clicks the app's name. Desktop: a background thread with `ureq`; web: `fetch` in palettecraft.js.

/// This build's version (from Cargo.toml).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub's description of the latest published release (JSON with `tag_name`).
pub const LATEST_RELEASE_API: &str = "https://api.github.com/repos/skrilic/PaletteCraft/releases/latest";

/// Project website, whose download buttons always point at the latest release (the web
/// app updates itself instead).
#[cfg(not(target_arch = "wasm32"))]
pub const DOWNLOAD_URL: &str = "https://skrilic.github.io/PaletteCraft/";

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Idle,
    Checking,
    Latest,
    /// A newer version is out (its number, without the "v").
    Newer(String),
    Failed(String),
}

pub struct UpdateCheck {
    pub state: State,
    #[cfg(not(target_arch = "wasm32"))]
    answer: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
}

impl UpdateCheck {
    pub fn new() -> Self {
        UpdateCheck {
            state: State::Idle,
            #[cfg(not(target_arch = "wasm32"))]
            answer: None,
        }
    }

    /// Starts asking GitHub, unless a question is already on its way.
    pub fn start(&mut self) {
        if self.state == State::Checking {
            return;
        }
        self.state = State::Checking;
        #[cfg(target_arch = "wasm32")]
        crate::web::check_update(LATEST_RELEASE_API);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (tx, rx) = std::sync::mpsc::channel();
            self.answer = Some(rx);
            std::thread::spawn(move || {
                let _ = tx.send(latest_tag());
            });
        }
    }

    /// Takes GitHub's answer once it has come (call every frame while `Checking`).
    pub fn poll(&mut self) {
        if self.state != State::Checking {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        let answer = crate::web::take_update();
        #[cfg(not(target_arch = "wasm32"))]
        let answer = match self.answer.as_ref().map(|rx| rx.try_recv()) {
            Some(Ok(a)) => Some(a),
            Some(Err(std::sync::mpsc::TryRecvError::Empty)) => None,
            _ => Some(Err("the check stopped unexpectedly".into())),
        };
        if let Some(answer) = answer {
            self.state = judge(answer);
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.answer = None;
            }
        }
    }
}

/// The state for GitHub's answer: the latest release's tag, or why it could not be read.
fn judge(answer: Result<String, String>) -> State {
    let tag = match answer {
        Ok(tag) => tag,
        Err(e) => return State::Failed(e),
    };
    match (parse_version(&tag), parse_version(VERSION)) {
        (Some(latest), Some(ours)) if latest > ours => State::Newer(tag.trim_start_matches('v').to_string()),
        (Some(_), Some(_)) => State::Latest,
        _ => State::Failed(format!("unexpected version \"{tag}\"")),
    }
}

/// "v0.3.1" or "0.3.1" as (major, minor, patch).
fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let mut parts = s.trim().trim_start_matches('v').split('.').map(|p| p.parse::<u32>().ok());
    let v = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(v)
}

/// Tag of the latest release, e.g. "v0.3.2" (desktop; blocks for up to 15 s).
#[cfg(not(target_arch = "wasm32"))]
fn latest_tag() -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(15)))
        // GitHub's API refuses requests without a User-Agent
        .user_agent(format!("PaletteCraft/{VERSION}"))
        .build()
        .into();
    let body = agent
        .get(LATEST_RELEASE_API)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    json["tag_name"].as_str().map(str::to_string).ok_or_else(|| "no version in GitHub's answer".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_by_number() {
        assert_eq!(parse_version("v0.3.1"), Some((0, 3, 1)));
        assert_eq!(parse_version("1.10.0"), Some((1, 10, 0)));
        assert_eq!(parse_version("v1.2"), None);
        assert_eq!(parse_version("v1.2.3-beta"), None);
        assert!(parse_version("v0.10.0") > parse_version("v0.9.9"));
        assert!(parse_version(VERSION).is_some());
    }

    #[test]
    fn newer_only_when_greater() {
        assert_eq!(judge(Ok(format!("v{VERSION}"))), State::Latest);
        assert_eq!(judge(Ok("v0.0.1".into())), State::Latest);
        assert_eq!(judge(Ok("v999.0.0".into())), State::Newer("999.0.0".into()));
        assert!(matches!(judge(Ok("nightly".into())), State::Failed(_)));
        assert!(matches!(judge(Err("offline".into())), State::Failed(_)));
    }

    /// Needs the network: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn github_answers_with_a_version() {
        let tag = latest_tag().unwrap();
        assert!(parse_version(&tag).is_some(), "{tag}");
    }
}

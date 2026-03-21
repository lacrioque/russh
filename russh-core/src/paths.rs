use std::env;
use std::path::PathBuf;

/// Returns the default config file path.
///
/// Resolution order:
/// 1. `XDG_CONFIG_HOME/russh/config.toml` (if `XDG_CONFIG_HOME` is set)
/// 2. `~/.config/russh/config.toml`
///
/// If `override_path` is `Some`, it is returned directly (after tilde expansion).
pub fn config_path(override_path: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = override_path {
        return Some(PathBuf::from(expand_tilde(p)));
    }

    let config_dir = if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        if xdg.is_empty() {
            default_config_dir()?
        } else {
            PathBuf::from(xdg)
        }
    } else {
        default_config_dir()?
    };

    Some(config_dir.join("russh").join("config.toml"))
}

/// Returns `~/.config` using the home directory, or `None` if unavailable.
fn default_config_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".config"))
}

/// Expands a leading `~` or `~/` to the user's home directory.
///
/// - `~/foo` → `/home/user/foo`
/// - `~` → `/home/user`
/// - Paths without a leading `~` are returned unchanged.
/// - `~otheruser/...` is **not** expanded (no NSS lookup).
///
/// If the home directory cannot be determined, the path is returned unchanged.
pub fn expand_tilde(path: &str) -> String {
    if path == "~" {
        return home_dir()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string());
    }

    if let Some(rest) = path.strip_prefix("~/") {
        return home_dir()
            .map(|h| h.join(rest).to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string());
    }

    path.to_string()
}

/// Returns the current user's home directory.
///
/// Uses `HOME` on Unix, `USERPROFILE` on Windows.
fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        env::var_os("HOME").map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        env::var_os("USERPROFILE")
            .or_else(|| env::var_os("HOME"))
            .map(PathBuf::from)
    }
    #[cfg(not(any(unix, windows)))]
    {
        env::var_os("HOME").map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// Helper: run a closure with specific env vars set, restoring originals afterward.
    fn with_env<F: FnOnce()>(vars: &[(&str, Option<&str>)], f: F) {
        let originals: Vec<_> = vars
            .iter()
            .map(|(k, _)| (*k, env::var_os(k)))
            .collect();

        for (k, v) in vars {
            match v {
                Some(val) => env::set_var(k, val),
                None => env::remove_var(k),
            }
        }

        f();

        for (k, original) in &originals {
            match original {
                Some(val) => env::set_var(k, val),
                None => env::remove_var(k),
            }
        }
    }

    #[test]
    fn expand_tilde_home() {
        with_env(&[("HOME", Some("/fakehome"))], || {
            assert_eq!(expand_tilde("~"), "/fakehome");
        });
    }

    #[test]
    fn expand_tilde_subpath() {
        with_env(&[("HOME", Some("/fakehome"))], || {
            assert_eq!(expand_tilde("~/.ssh/id_rsa"), "/fakehome/.ssh/id_rsa");
        });
    }

    #[test]
    fn expand_tilde_no_prefix() {
        assert_eq!(expand_tilde("/absolute/path"), "/absolute/path");
        assert_eq!(expand_tilde("relative/path"), "relative/path");
    }

    #[test]
    fn expand_tilde_other_user_unchanged() {
        assert_eq!(expand_tilde("~bob/.ssh/key"), "~bob/.ssh/key");
    }

    #[test]
    fn config_path_override() {
        with_env(&[("HOME", Some("/fakehome"))], || {
            let p = config_path(Some("~/myconfig.toml")).unwrap();
            assert_eq!(p, PathBuf::from("/fakehome/myconfig.toml"));
        });
    }

    #[test]
    fn config_path_override_absolute() {
        let p = config_path(Some("/etc/russh.toml")).unwrap();
        assert_eq!(p, PathBuf::from("/etc/russh.toml"));
    }

    #[test]
    fn config_path_default() {
        with_env(
            &[("HOME", Some("/fakehome")), ("XDG_CONFIG_HOME", None)],
            || {
                let p = config_path(None).unwrap();
                assert_eq!(
                    p,
                    PathBuf::from("/fakehome/.config/russh/config.toml")
                );
            },
        );
    }

    #[test]
    fn config_path_xdg() {
        with_env(
            &[
                ("HOME", Some("/fakehome")),
                ("XDG_CONFIG_HOME", Some("/xdgdir")),
            ],
            || {
                let p = config_path(None).unwrap();
                assert_eq!(p, PathBuf::from("/xdgdir/russh/config.toml"));
            },
        );
    }

    #[test]
    fn expand_tilde_no_home_returns_path_unchanged() {
        with_env(&[("HOME", None)], || {
            // Without HOME we cannot expand; original string is returned.
            let result = expand_tilde("~/foo");
            assert_eq!(result, "~/foo");
        });
    }

    #[test]
    fn expand_tilde_bare_no_home_returns_tilde() {
        with_env(&[("HOME", None)], || {
            let result = expand_tilde("~");
            assert_eq!(result, "~");
        });
    }

    #[test]
    fn expand_tilde_empty_string() {
        assert_eq!(expand_tilde(""), "");
    }

    #[test]
    fn config_path_xdg_empty_falls_back() {
        with_env(
            &[
                ("HOME", Some("/fakehome")),
                ("XDG_CONFIG_HOME", Some("")),
            ],
            || {
                let p = config_path(None).unwrap();
                assert_eq!(
                    p,
                    PathBuf::from("/fakehome/.config/russh/config.toml")
                );
            },
        );
    }
}

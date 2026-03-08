pub fn abbreviate_home(path: &str, home_dir: Option<&str>) -> String {
    match home_dir {
        Some(home) => {
            let home_prefix = format!("{home}/");
            if path == home {
                "~".to_string()
            } else if let Some(rest) = path.strip_prefix(&home_prefix) {
                format!("~/{rest}")
            } else {
                path.to_string()
            }
        }
        None => path.to_string(),
    }
}

pub fn expand_home(path: &str, home_dir: Option<&str>) -> String {
    match home_dir {
        Some(home) => {
            if path == "~" {
                home.to_string()
            } else if let Some(rest) = path.strip_prefix("~/") {
                format!("{home}/{rest}")
            } else {
                path.to_string()
            }
        }
        None => path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = "/Users/testuser";

    #[test]
    fn abbreviates_home_prefix() {
        assert_eq!(
            abbreviate_home("/Users/testuser/src/project", Some(HOME)),
            "~/src/project"
        );
    }

    #[test]
    fn abbreviates_home_dir_itself() {
        assert_eq!(abbreviate_home("/Users/testuser", Some(HOME)), "~");
    }

    #[test]
    fn leaves_non_home_paths_unchanged() {
        assert_eq!(
            abbreviate_home("/usr/local/bin", Some(HOME)),
            "/usr/local/bin"
        );
    }

    #[test]
    fn leaves_similar_prefix_paths_unchanged() {
        assert_eq!(
            abbreviate_home("/Users/testuserextra/project", Some(HOME)),
            "/Users/testuserextra/project"
        );
    }

    #[test]
    fn leaves_relative_paths_unchanged() {
        assert_eq!(abbreviate_home("src/main.rs", Some(HOME)), "src/main.rs");
    }

    #[test]
    fn abbreviate_home_none_returns_path() {
        assert_eq!(
            abbreviate_home("/Users/testuser/src/project", None),
            "/Users/testuser/src/project"
        );
    }

    #[test]
    fn expand_home_tilde_prefix() {
        assert_eq!(
            expand_home("~/src/proj", Some(HOME)),
            "/Users/testuser/src/proj"
        );
    }

    #[test]
    fn expand_home_tilde_alone() {
        assert_eq!(expand_home("~", Some(HOME)), "/Users/testuser");
    }

    #[test]
    fn expand_home_absolute_unchanged() {
        assert_eq!(expand_home("/usr/local", Some(HOME)), "/usr/local");
    }

    #[test]
    fn expand_home_relative_unchanged() {
        assert_eq!(expand_home("src/main.rs", Some(HOME)), "src/main.rs");
    }

    #[test]
    fn expand_home_tilde_in_middle_unchanged() {
        assert_eq!(expand_home("/foo/~bar", Some(HOME)), "/foo/~bar");
    }

    #[test]
    fn expand_home_tilde_user_unchanged() {
        assert_eq!(expand_home("~bob/foo", Some(HOME)), "~bob/foo");
    }

    #[test]
    fn expand_home_none_returns_path() {
        assert_eq!(expand_home("~/src/proj", None), "~/src/proj");
    }

    #[test]
    fn expand_and_abbreviate_roundtrip() {
        let absolute = "/Users/testuser/src/proj";
        let home = Some(HOME);
        assert_eq!(
            expand_home(&abbreviate_home(absolute, home), home),
            absolute
        );
    }
}

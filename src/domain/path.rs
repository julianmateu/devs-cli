pub fn abbreviate_home(path: &str) -> String {
    match dirs::home_dir() {
        Some(home) => {
            let home_str = home.to_string_lossy();
            let home_prefix = format!("{home_str}/");
            if path == home_str.as_ref() {
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

pub fn expand_home(path: &str) -> String {
    match dirs::home_dir() {
        Some(home) => {
            let home_str = home.to_string_lossy();
            if path == "~" {
                home_str.into_owned()
            } else if let Some(rest) = path.strip_prefix("~/") {
                format!("{home_str}/{rest}")
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

    #[test]
    fn abbreviates_home_prefix() {
        let home = dirs::home_dir().unwrap();
        let path = format!("{}/src/project", home.display());
        assert_eq!(abbreviate_home(&path), "~/src/project");
    }

    #[test]
    fn abbreviates_home_dir_itself() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(abbreviate_home(&home.to_string_lossy()), "~");
    }

    #[test]
    fn leaves_non_home_paths_unchanged() {
        assert_eq!(abbreviate_home("/usr/local/bin"), "/usr/local/bin");
    }

    #[test]
    fn leaves_similar_prefix_paths_unchanged() {
        let home = dirs::home_dir().unwrap();
        let path = format!("{}extra/project", home.display());
        assert_eq!(abbreviate_home(&path), path);
    }

    #[test]
    fn leaves_relative_paths_unchanged() {
        assert_eq!(abbreviate_home("src/main.rs"), "src/main.rs");
    }

    #[test]
    fn expand_home_tilde_prefix() {
        let home = dirs::home_dir().unwrap();
        let expected = format!("{}/src/proj", home.display());
        assert_eq!(expand_home("~/src/proj"), expected);
    }

    #[test]
    fn expand_home_tilde_alone() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_home("~"), home.to_string_lossy().as_ref());
    }

    #[test]
    fn expand_home_absolute_unchanged() {
        assert_eq!(expand_home("/usr/local"), "/usr/local");
    }

    #[test]
    fn expand_home_relative_unchanged() {
        assert_eq!(expand_home("src/main.rs"), "src/main.rs");
    }

    #[test]
    fn expand_home_tilde_in_middle_unchanged() {
        assert_eq!(expand_home("/foo/~bar"), "/foo/~bar");
    }

    #[test]
    fn expand_home_tilde_user_unchanged() {
        assert_eq!(expand_home("~bob/foo"), "~bob/foo");
    }

    #[test]
    fn expand_and_abbreviate_roundtrip() {
        let home = dirs::home_dir().unwrap();
        let absolute = format!("{}/src/proj", home.display());
        assert_eq!(expand_home(&abbreviate_home(&absolute)), absolute);
    }
}

pub fn abbreviate_home(path: &str) -> String {
    match dirs::home_dir() {
        Some(home) => {
            let home_str = home.to_string_lossy();
            if path == home_str.as_ref() {
                "~".to_string()
            } else if let Some(rest) = path.strip_prefix(home_str.as_ref()) {
                format!("~{rest}")
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
    fn leaves_relative_paths_unchanged() {
        assert_eq!(abbreviate_home("src/main.rs"), "src/main.rs");
    }
}

fn home_dir_string() -> Option<String> {
    dirs::home_dir().map(|p| p.to_string_lossy().into_owned())
}

pub fn abbreviate_home(path: &str) -> String {
    crate::domain::path::abbreviate_home(path, home_dir_string().as_deref())
}

pub fn expand_home(path: &str) -> String {
    crate::domain::path::expand_home(path, home_dir_string().as_deref())
}

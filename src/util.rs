pub fn normalize_string(s: &str) -> String {
    s.replace("-", "").replace("_", "").to_lowercase()
}
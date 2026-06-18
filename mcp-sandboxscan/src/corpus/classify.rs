/// Heuristic WASM portability class for a GitHub primary language label.
pub fn wasm_class_from_language(lang: Option<&str>) -> &'static str {
    match lang.unwrap_or("").to_lowercase().as_str() {
        "rust" | "go" => "wasm-ready",
        "python" => "wasm-needs-runtime",
        "javascript" | "typescript" => "wasm-hard",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_ecosystems() {
        assert_eq!(wasm_class_from_language(Some("Rust")), "wasm-ready");
        assert_eq!(wasm_class_from_language(Some("Python")), "wasm-needs-runtime");
        assert_eq!(wasm_class_from_language(Some("TypeScript")), "wasm-hard");
        assert_eq!(wasm_class_from_language(None), "unknown");
    }
}

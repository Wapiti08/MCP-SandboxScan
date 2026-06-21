use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::subject::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ecosystem {
    Npm,
    Go,
    Python,
    Rust,
    Unknown,
}

impl Ecosystem {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Go => "go",
            Self::Python => "python",
            Self::Rust => "rust",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_language(lang: &Language) -> Self {
        match lang {
            Language::TypeScript | Language::JavaScript => Self::Npm,
            Language::Go => Self::Go,
            Language::Python => Self::Python,
            Language::Rust => Self::Rust,
            _ => Self::Unknown,
        }
    }
}

pub fn count_dependencies(root: &Path, lang: Language) -> u32 {
    match lang {
        Language::TypeScript | Language::JavaScript => count_npm_deps(root),
        Language::Go => count_go_requires(root),
        Language::Python => count_python_deps(root),
        Language::Rust => count_cargo_deps(root),
        _ => 0,
    }
}

fn count_npm_deps(root: &Path) -> u32 {
    let path = root.join("package.json");
    let Ok(raw) = fs::read_to_string(path) else {
        return 0;
    };
    let Ok(pkg) = serde_json::from_str::<Value>(&raw) else {
        return 0;
    };
    let mut n = 0u32;
    for key in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        if let Some(obj) = pkg.get(key).and_then(|v| v.as_object()) {
            n += obj.len() as u32;
        }
    }
    n
}

fn count_go_requires(root: &Path) -> u32 {
    let path = root.join("go.mod");
    let Ok(raw) = fs::read_to_string(path) else {
        return 0;
    };
    raw.lines()
        .filter(|line| {
            let t = line.trim();
            t.starts_with("require ") || (t.starts_with('\t') && !t.contains("// indirect"))
        })
        .count() as u32
}

fn count_python_deps(root: &Path) -> u32 {
    if let Ok(raw) = fs::read_to_string(root.join("requirements.txt")) {
        let n = raw
            .lines()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with('#')
            })
            .count() as u32;
        if n > 0 {
            return n;
        }
    }
    let path = root.join("pyproject.toml");
    let Ok(raw) = fs::read_to_string(path) else {
        return 0;
    };
    raw.lines()
        .skip_while(|l| !l.trim().starts_with('[') || !l.contains("dependencies"))
        .skip(1)
        .take_while(|l| !l.trim().starts_with('['))
        .filter(|l| l.contains('='))
        .count() as u32
}

fn count_cargo_deps(root: &Path) -> u32 {
    let path = root.join("Cargo.toml");
    let Ok(raw) = fs::read_to_string(path) else {
        return 0;
    };
    raw.lines()
        .skip_while(|l| !l.trim().eq("[dependencies]"))
        .skip(1)
        .take_while(|l| !l.trim().starts_with('['))
        .filter(|l| l.contains('='))
        .count() as u32
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn counts_playwright_npm_deps() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("corpus/clones/microsoft__playwright-mcp");
        if !root.exists() {
            return;
        }
        assert!(count_dependencies(&root, Language::TypeScript) > 0);
    }
}

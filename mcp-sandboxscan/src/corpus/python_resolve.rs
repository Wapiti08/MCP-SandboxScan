use std::fs;
use std::path::Path;

use anyhow::{Result, bail};

/// Find a plausible stdio MCP entrypoint for Python repos.
pub fn find_python_entrypoint(root: &Path) -> Result<String> {
    for rel in [
        "server.py",
        "main.py",
        "src/server.py",
        "src/main.py",
        "__main__.py",
        "src/__main__.py",
        "mcp_server.py",
        "src/mcp_server.py",
    ] {
        if root.join(rel).is_file() {
            return Ok(rel.to_string());
        }
    }

    if let Some(path) = find_from_pyproject_scripts(root)? {
        return Ok(path);
    }

    if let Some(path) = find_in_shallow_packages(root, 2)? {
        return Ok(path);
    }

    bail!(
        "no python MCP entrypoint under {} (tried server.py/main.py, pyproject scripts, packages)",
        root.display()
    );
}

fn find_from_pyproject_scripts(root: &Path) -> Result<Option<String>> {
    let path = root.join("pyproject.toml");
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    let mut in_scripts = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_scripts = trimmed == "[project.scripts]" || trimmed == "[tool.poetry.scripts]";
            continue;
        }
        if !in_scripts || !trimmed.contains('=') {
            continue;
        }
        let Some((_, rhs)) = trimmed.split_once('=') else {
            continue;
        };
        let target = rhs.trim().trim_matches('"').trim_matches('\'');
        if let Some(rel) = module_target_to_path(root, target) {
            return Ok(Some(rel));
        }
    }
    Ok(None)
}

fn module_target_to_path(root: &Path, target: &str) -> Option<String> {
    let module = target.split(':').next()?.replace('.', "/");
    for suffix in [".py", "/__main__.py"] {
        let rel = if suffix.starts_with('/') {
            format!("{module}{suffix}")
        } else {
            format!("{module}{suffix}")
        };
        if root.join(&rel).is_file() {
            return Some(rel);
        }
    }
    None
}

fn find_in_shallow_packages(root: &Path, max_depth: u32) -> Result<Option<String>> {
    find_in_dir(root, root, 0, max_depth)
}

fn find_in_dir(root: &Path, dir: &Path, depth: u32, max_depth: u32) -> Result<Option<String>> {
    if depth > max_depth {
        return Ok(None);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if should_skip_dir(&name) {
                continue;
            }
            if let Some(found) = find_in_dir(root, &path, depth + 1, max_depth)? {
                return Ok(Some(found));
            }
            continue;
        }

        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !matches!(
            name,
            "server.py" | "main.py" | "__main__.py" | "mcp_server.py"
        ) {
            continue;
        }
        if looks_like_mcp_server_source(&fs::read_to_string(&path).unwrap_or_default()) {
            return Ok(Some(path.strip_prefix(root)?.display().to_string()));
        }
    }
    Ok(None)
}

fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".venv"
            | "venv"
            | "node_modules"
            | "dist"
            | "build"
            | "tests"
            | "test"
            | "docs"
            | "examples"
            | "scripts"
            | ".github"
    ) || name.starts_with('.')
}

fn looks_like_mcp_server_source(raw: &str) -> bool {
    raw.contains("fastmcp")
        || raw.contains("mcp.server")
        || raw.contains("MCPServer")
        || raw.contains("stdio_server")
        || raw.contains("from mcp")
}

pub fn resolve_python_manifest(root: &Path, slug: &str) -> Result<String> {
    let entry = find_python_entrypoint(root)?;
    let source_dir = root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .display()
        .to_string();

    Ok(format!(
        r#"name = "{slug}"
language = "python"
source_dir = "{source_dir}"
entrypoint = "{entry}"
capabilities = ["stdio", "mcp-protocol"]

[build]
command = "bash"
args = ["-c", "python3 -m venv .venv && (.venv/bin/pip install -e . -q || .venv/bin/pip install -r requirements.txt -q)"]

[run]
command = ".venv/bin/python"
args = ["-u", "{entry}"]

[mcp]
tool = "echo"
arguments = {{}}
"#
    ))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn finds_datagouv_main() {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/clones/datagouv__datagouv-mcp");
        if !root.exists() {
            return;
        }
        let entry = find_python_entrypoint(&root).expect("entrypoint");
        assert!(entry.contains("main.py") || entry.contains("server.py"));
    }
}

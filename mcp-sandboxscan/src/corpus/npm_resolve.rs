use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct NpmPackagePlan {
    pub package_dir: PathBuf,
    pub install_dir: PathBuf,
    pub run_command: String,
    pub run_args: Vec<String>,
}

pub fn resolve_npm(root: &Path, slug: &str) -> Result<String> {
    let plan = plan_npm_package(root)?;
    let package_dir = plan
        .package_dir
        .canonicalize()
        .unwrap_or(plan.package_dir.clone());
    let install_dir = plan
        .install_dir
        .canonicalize()
        .unwrap_or(plan.install_dir.clone());

    let build_args = npm_build_args(&install_dir, &package_dir);

    let run_args = serde_json::to_string(&plan.run_args)?;

    Ok(format!(
        r#"name = "{slug}"
language = "type-script"
source_dir = "{package_dir}"
capabilities = ["stdio", "mcp-protocol"]

[build]
command = "bash"
args = {build_args}

[run]
command = "{run_command}"
args = {run_args}

[mcp]
tool = "echo"
arguments = {{}}
"#,
        package_dir = package_dir.display(),
        run_command = plan.run_command,
    ))
}

pub fn plan_npm_package(root: &Path) -> Result<NpmPackagePlan> {
    let root_pkg_path = root.join("package.json");
    if !root_pkg_path.exists() {
        bail!("no package.json under {}", root.display());
    }

    let root_pkg = read_package_json(&root_pkg_path)?;
    let workspace_dirs = workspace_package_dirs(root, &root_pkg)?;

    let candidates: Vec<PathBuf> = if workspace_dirs.is_empty() {
        vec![root.to_path_buf()]
    } else {
        workspace_dirs
    };

    let mut best: Option<(i32, PathBuf, Value)> = None;
    for dir in candidates {
        let pkg_path = dir.join("package.json");
        if !pkg_path.exists() {
            continue;
        }
        let pkg = read_package_json(&pkg_path)?;
        let score = score_npm_package(&dir, &pkg);
        if score < 0 {
            continue;
        }
        match &best {
            Some((best_score, _, _)) if score <= *best_score => {}
            _ => best = Some((score, dir, pkg)),
        }
    }

    let (_, package_dir, pkg) = best.ok_or_else(|| {
        anyhow::anyhow!("no runnable npm workspace/package found under {}", root.display())
    })?;

    let install_dir = if workspace_package_dirs(root, &root_pkg)?.is_empty() {
        package_dir.clone()
    } else {
        root.to_path_buf()
    };

    let (run_command, run_args) = npm_run_spec(&pkg)?;
    let run_command = normalize_run_command(&run_command, &run_args);
    let run_args = npm_stdio_run_args(&pkg, &run_args);

    Ok(NpmPackagePlan {
        package_dir,
        install_dir,
        run_command,
        run_args,
    })
}

fn read_package_json(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn workspace_package_dirs(root: &Path, pkg: &Value) -> Result<Vec<PathBuf>> {
    let Some(workspaces) = pkg.get("workspaces") else {
        return Ok(vec![]);
    };

    let globs: Vec<String> = match workspaces {
        Value::Array(items) => items
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        Value::Object(obj) => obj
            .get("packages")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        _ => vec![],
    };

    let mut dirs = Vec::new();
    for glob in globs {
        expand_workspace_glob(root, &glob, &mut dirs)?;
    }
    dirs.sort();
    dirs.dedup();
    Ok(dirs)
}

fn expand_workspace_glob(root: &Path, glob: &str, out: &mut Vec<PathBuf>) -> Result<()> {
    let pattern = glob.replace('\\', "/");
    if pattern.ends_with("/*") {
        let parent = root.join(pattern.trim_end_matches("/*"));
        if parent.is_dir() {
            for entry in fs::read_dir(&parent)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    if entry.path().join("package.json").exists() {
                        out.push(entry.path());
                    }
                }
            }
        }
        return Ok(());
    }

    let path = root.join(&pattern);
    if path.join("package.json").exists() {
        out.push(path);
    }
    Ok(())
}

fn score_npm_package(dir: &Path, pkg: &Value) -> i32 {
    let name = pkg
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    let dir_name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut score = 0;

    if pkg.get("bin").is_some() {
        score += 10;
    }
    if pkg.get("scripts")
        .and_then(|s| s.get("start:stdio"))
        .is_some()
    {
        score += 8;
    }
    if name.contains("@modelcontextprotocol/") || name.contains("mcp-server") {
        score += 12;
    }
    if name.contains("mcp") || dir_name.contains("mcp") {
        score += 4;
    }
    if dir_name.contains("server") || name.contains("server") {
        score += 3;
    }
    if pkg.get("mcpName").is_some() {
        score += 5;
    }
    if pkg.get("dependencies")
        .or(pkg.get("devDependencies"))
        .and_then(|deps| deps.get("@modelcontextprotocol/sdk"))
        .is_some()
    {
        score += 4;
    }

    // Penalize test-only or meta packages.
    if dir_name == "scripts" || dir_name == "docs" || name.ends_with("-cli") && !name.contains("mcp")
    {
        score -= 20;
    }
    if score == 0 && pkg.get("bin").is_none() && pkg.get("main").is_none() {
        score = -1;
    }
    score
}

fn npm_run_spec(pkg: &Value) -> Result<(String, Vec<String>)> {
    if let Some(script) = pkg
        .get("scripts")
        .and_then(|s| s.get("start:stdio"))
        .and_then(|v| v.as_str())
    {
        if let Some((cmd, args)) = split_shell_command(script) {
            return Ok((cmd, args));
        }
    }

    if let Some(script) = pkg
        .get("scripts")
        .and_then(|s| s.get("start"))
        .and_then(|v| v.as_str())
    {
        if script.contains("node ") || script.ends_with(".js") {
            if let Some((cmd, args)) = split_shell_command(script) {
                return Ok((cmd, args));
            }
        }
    }

    if let Some(bin) = pkg.get("bin") {
        match bin {
            Value::Object(map) => {
                if let Some((_, path)) = map.iter().next() {
                    let path = path.as_str().context("bin path must be string")?;
                    return Ok(("node".to_string(), vec![path.to_string()]));
                }
            }
            Value::String(path) => {
                return Ok(("node".to_string(), vec![path.clone()]));
            }
            _ => {}
        }
    }

    if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
        return Ok(("node".to_string(), vec![main.to_string()]));
    }

    bail!("no npm bin/main/start entrypoint found");
}

fn normalize_run_command(command: &str, args: &[String]) -> String {
    if command == "bun" && args.first().is_some_and(|a| a.ends_with(".js")) {
        return "node".to_string();
    }
    if command == "pnpm" || command == "yarn" {
        return "node".to_string();
    }
    command.to_string()
}

fn npm_build_args(install_dir: &Path, package_dir: &Path) -> String {
    let install = install_dir.to_string_lossy().replace('\'', r"'\''");
    let package = package_dir.to_string_lossy().replace('\'', r"'\''");
    let script = if install_dir == package_dir {
        "npm install --no-fund --no-audit && npm run build --if-present".to_string()
    } else {
        format!(
            "cd '{install}' && npm install --no-fund --no-audit && cd '{package}' && npm run build --if-present"
        )
    };
    serde_json::to_string(&["-c", script.as_str()]).unwrap_or_else(|_| {
        r#"["-c", "npm install --no-fund --no-audit"]"#.to_string()
    })
}

fn npm_stdio_run_args(pkg: &Value, run_args: &[String]) -> Vec<String> {
    let mut args = run_args.to_vec();
    if args
        .iter()
        .any(|a| a == "stdio" || a == "--stdio")
    {
        return args;
    }

    let needs_stdio = pkg
        .get("scripts")
        .and_then(|s| s.as_object())
        .is_some_and(|scripts| {
            scripts.values().filter_map(|v| v.as_str()).any(|script| {
                script.contains("--stdio")
                    || script.contains(" stdio")
                    || script.contains("start:stdio")
            })
        });

    if needs_stdio {
        args.push("--stdio".to_string());
    }
    args
}

fn split_shell_command(script: &str) -> Option<(String, Vec<String>)> {
    let script = script.trim();
    if script.is_empty() {
        return None;
    }
    let mut parts = script.split_whitespace();
    let cmd = parts.next()?.to_string();
    let args = parts.map(|s| s.to_string()).collect();
    Some((cmd, args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_playwright_mcp_bin() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("corpus/clones/microsoft__playwright-mcp");
        if !root.exists() {
            return;
        }
        let plan = plan_npm_package(&root).expect("plan playwright");
        assert_eq!(plan.run_command, "node");
        assert_eq!(plan.run_args, vec!["cli.js".to_string()]);
    }

    #[test]
    fn plans_mcp_servers_workspace() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("corpus/clones/modelcontextprotocol__servers");
        if !root.exists() {
            return;
        }
        let plan = plan_npm_package(&root).expect("plan servers monorepo");
        assert!(plan
            .package_dir
            .to_string_lossy()
            .contains("everything"));
        assert_eq!(plan.run_command, "node");
        assert!(plan.run_args.iter().any(|a| a.contains("index.js")));
    }

    #[test]
    fn split_node_stdio_script() {
        let (cmd, args) = split_shell_command("node dist/index.js stdio").unwrap();
        assert_eq!(cmd, "node");
        assert_eq!(args, vec!["dist/index.js", "stdio"]);
    }
}

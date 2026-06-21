use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone)]
pub struct GoPackagePlan {
    pub source_dir: PathBuf,
    pub build_args: Vec<String>,
    pub binary: String,
    pub run_args: Vec<String>,
}

pub fn resolve_go(root: &Path, slug: &str) -> Result<String> {
    let plan = plan_go_package(root)?;
    let source_dir = plan
        .source_dir
        .canonicalize()
        .unwrap_or(plan.source_dir.clone());

    let build_args = serde_json::to_string(&plan.build_args)?;

    let run_args = serde_json::to_string(&plan.run_args)?;

    Ok(format!(
        r#"name = "{slug}"
language = "go"
source_dir = "{source_dir}"
capabilities = ["stdio", "mcp-protocol"]

[build]
command = "go"
args = {build_args}

[run]
command = "{binary}"
args = {run_args}

[mcp]
tool = "echo"
arguments = {{}}
"#,
        source_dir = source_dir.display(),
        binary = plan.binary,
    ))
}

pub fn infer_go_run_args(root: &Path) -> Vec<String> {
    if go_source_has_stdio_subcommand(root) {
        vec!["stdio".to_string()]
    } else {
        vec![]
    }
}

fn go_source_has_stdio_subcommand(root: &Path) -> bool {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if dir.components().count() - root.components().count() < 4 {
                    stack.push(path);
                }
                continue;
            }
            if path.extension().is_none_or(|ext| ext != "go") {
                continue;
            }
            let Ok(raw) = fs::read_to_string(&path) else {
                continue;
            };
            if raw.contains("Use:") && raw.contains("\"stdio\"") {
                return true;
            }
        }
    }
    false
}

pub fn plan_go_package(root: &Path) -> Result<GoPackagePlan> {
    if !root.join("go.mod").exists() {
        bail!("no go.mod under {}", root.display());
    }

    if root.join("main.go").exists() && is_main_package(&root.join("main.go"))? {
        return Ok(GoPackagePlan {
            source_dir: root.to_path_buf(),
            build_args: vec![
                "build".to_string(),
                "-o".to_string(),
                "server".to_string(),
                ".".to_string(),
            ],
            binary: "./server".to_string(),
            run_args: infer_go_run_args(root),
        });
    }

    let cmd_dir = root.join("cmd");
    if !cmd_dir.is_dir() {
        bail!("no main.go or cmd/ package under {}", root.display());
    }

    let mut best: Option<(i32, PathBuf)> = None;
    for entry in fs::read_dir(&cmd_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let main_go = entry.path().join("main.go");
        if !main_go.exists() || !is_main_package(&main_go)? {
            continue;
        }
        let score = score_go_cmd(entry.file_name().to_string_lossy().as_ref());
        match &best {
            Some((best_score, _)) if score <= *best_score => {}
            _ => best = Some((score, entry.path())),
        }
    }

    let cmd_path = best
        .map(|(_, p)| p)
        .ok_or_else(|| anyhow::anyhow!("no cmd/*/main.go package under {}", root.display()))?;

    let rel = path_relative_to(root, &cmd_path)?;
    let run_args = infer_go_run_args(root);
    Ok(GoPackagePlan {
        source_dir: root.to_path_buf(),
        build_args: vec![
            "build".to_string(),
            "-o".to_string(),
            "server".to_string(),
            rel,
        ],
        binary: "./server".to_string(),
        run_args,
    })
}

fn is_main_package(main_go: &Path) -> Result<bool> {
    let raw = fs::read_to_string(main_go).with_context(|| format!("read {}", main_go.display()))?;
    Ok(raw.lines().any(|line| line.trim() == "package main"))
}

fn score_go_cmd(name: &str) -> i32 {
    let lower = name.to_lowercase();
    let mut score = 0;
    if lower.contains("mcp-server") || lower == "github-mcp-server" {
        score += 20;
    }
    if lower.ends_with("-mcp") || lower.contains("mcp") {
        score += 10;
    }
    if lower.contains("server") {
        score += 8;
    }
    if lower == "server" {
        score += 5;
    }
    if lower.contains("curl") || lower.contains("test") || lower.contains("script") {
        score -= 15;
    }
    score
}

fn path_relative_to(root: &Path, target: &Path) -> Result<String> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
    let rel = target
        .strip_prefix(&root)
        .with_context(|| format!("{} not under {}", target.display(), root.display()))?;
    Ok(format!("./{}", rel.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_github_mcp_server_cmd() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("corpus/clones/github__github-mcp-server");
        if !root.exists() {
            return;
        }
        let plan = plan_go_package(&root).expect("plan github mcp server");
        assert!(
            plan.build_args
                .iter()
                .any(|a| a.contains("github-mcp-server"))
        );
        assert_eq!(plan.run_args, vec!["stdio".to_string()]);
    }

    #[test]
    fn plans_mcp_toolbox_root_main() {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/clones/googleapis__mcp-toolbox");
        if !root.exists() {
            return;
        }
        let plan = plan_go_package(&root).expect("plan toolbox");
        assert_eq!(plan.build_args.last().map(String::as_str), Some("."));
    }
}

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::subject::Language;

use super::go_resolve;
use super::model::{CorpusFile, RepoEntry};
use super::npm_resolve;
use super::deps::{self, Ecosystem};

pub struct ResolveOptions {
    pub corpus_dir: PathBuf,
    pub max_repos: Option<usize>,
    pub skip_existing: bool,
}

pub fn resolve_corpus(corpus: &mut CorpusFile, opts: &ResolveOptions) -> Result<()> {
    let clones = opts.corpus_dir.join("clones");
    let manifests = opts.corpus_dir.join("manifests");
    fs::create_dir_all(&clones)?;
    fs::create_dir_all(&manifests)?;

    let mut n = 0usize;
    for repo in &mut corpus.repos {
        if let Some(max) = opts.max_repos {
            if n >= max {
                break;
            }
        }
        if opts.skip_existing && repo.resolved {
            continue;
        }

        match resolve_one(repo, &clones, &manifests) {
            Ok((toml_path, dest)) => {
                let lang = detect_language(&dest, repo.language.as_deref());
                repo.resolved = true;
                repo.scan_status = "resolved".into();
                repo.subject_toml = Some(toml_path);
                repo.resolve_error = None;
                repo.ecosystem = Ecosystem::from_language(&lang).as_str().to_string();
                repo.dep_count = deps::count_dependencies(&dest, lang);
                n += 1;
            }
            Err(err) => {
                repo.resolved = false;
                repo.scan_status = "skipped".into();
                repo.resolve_error = Some(err.to_string());
            }
        }
    }
    Ok(())
}

fn resolve_one(repo: &RepoEntry, clones: &Path, manifests: &Path) -> Result<(String, PathBuf)> {
    let slug = repo.id.replace('/', "__");
    let dest = clones.join(&slug);
    if !dest.exists() {
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                &repo.clone_url,
                &dest.to_string_lossy(),
            ])
            .status()
            .context("git clone")?;
        if !status.success() {
            bail!("git clone failed for {}", repo.id);
        }
    }

    let lang = detect_language(&dest, repo.language.as_deref());
    let toml = match lang {
        Language::TypeScript | Language::JavaScript => npm_resolve::resolve_npm(&dest, &slug)?,
        Language::Python => resolve_python(&dest, &slug)?,
        Language::Rust => resolve_rust(&dest, &slug)?,
        Language::Go => go_resolve::resolve_go(&dest, &slug)?,
        _ => bail!("unsupported language for {}", repo.id),
    };

    let out = manifests.join(format!("{slug}.toml"));
    fs::write(&out, toml)?;
    Ok((out.to_string_lossy().into_owned(), dest))
}

fn detect_language(root: &Path, github_lang: Option<&str>) -> Language {
    if root.join("package.json").exists() {
        return Language::TypeScript;
    }
    if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
        return Language::Python;
    }
    if root.join("Cargo.toml").exists() {
        return Language::Rust;
    }
    if root.join("go.mod").exists() {
        return Language::Go;
    }
    match github_lang.unwrap_or("").to_lowercase().as_str() {
        "rust" => Language::Rust,
        "go" => Language::Go,
        "python" => Language::Python,
        "typescript" | "javascript" => Language::TypeScript,
        _ => Language::Unknown,
    }
}

fn resolve_python(root: &Path, slug: &str) -> Result<String> {
    let dir = root.to_string_lossy();
    let entry = if root.join("server.py").exists() {
        "server.py"
    } else if root.join("src").join("server.py").exists() {
        "src/server.py"
    } else if root.join("main.py").exists() {
        "main.py"
    } else {
        bail!("no obvious python entrypoint under {dir}");
    };

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

fn resolve_rust(root: &Path, slug: &str) -> Result<String> {
    let bin = guess_cargo_bin_name(root).unwrap_or_else(|| slug.replace("__", "-"));
    let source_dir = root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .display()
        .to_string();

    Ok(format!(
        r#"name = "{slug}"
language = "rust"
source_dir = "{source_dir}"
capabilities = ["stdio", "mcp-protocol"]

[build]
command = "cargo"
args = ["build", "--release"]

[run]
command = "target/release/{bin}"
args = []

[mcp]
tool = "list_allowed_directories"
arguments = {{}}
"#
    ))
}

fn guess_cargo_bin_name(root: &Path) -> Option<String> {
    let raw = fs::read_to_string(root.join("Cargo.toml")).ok()?;
    for line in raw.lines() {
        if line.trim().starts_with("name = ") {
            return line.split('"').nth(1).map(|s| s.to_string());
        }
    }
    None
}

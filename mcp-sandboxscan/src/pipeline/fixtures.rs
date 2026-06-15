use std::path::Path;

use crate::subject::SubjectManifest;

pub fn ensure_fastmcp_examples(manifest_dir: &Path) {
    let marker = manifest_dir.join("external/fastmcp/examples/simple_echo.py");
    if marker.exists() {
        return;
    }

    let script = manifest_dir.join("scripts/fetch-fastmcp-examples.sh");
    assert!(
        script.exists(),
        "missing fetch script: {}",
        script.display()
    );

    let status = std::process::Command::new("bash")
        .arg(&script)
        .current_dir(manifest_dir)
        .status()
        .expect("run fetch-fastmcp-examples.sh");
    assert!(status.success(), "fetch-fastmcp-examples.sh failed");
    assert!(marker.exists(), "fastmcp examples missing after fetch");
}

pub fn ensure_python_venv(subject: &SubjectManifest, module: &str) {
    let venv_python = subject.source_dir.join(".venv/bin/python");
    if venv_python.exists() {
        let check = format!("import {module}");
        let ok = std::process::Command::new(&venv_python)
            .args(["-c", &check])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if ok {
            return;
        }
    }

    let Some(build) = &subject.build else {
        return;
    };

    let status = std::process::Command::new(&build.command)
        .args(&build.args)
        .current_dir(&subject.source_dir)
        .status()
        .expect("build python native MCP venv");
    assert!(status.success(), "python native MCP venv build failed");
}

pub fn ensure_python_fastmcp_venv(subject: &SubjectManifest) {
    ensure_python_venv(subject, "fastmcp");
}

pub fn ensure_go_sdk_examples(manifest_dir: &Path) {
    let marker = manifest_dir.join("external/go-sdk/examples/server/hello/main.go");
    if marker.exists() {
        return;
    }

    let script = manifest_dir.join("scripts/fetch-go-sdk-examples.sh");
    assert!(
        script.exists(),
        "missing fetch script: {}",
        script.display()
    );

    let status = std::process::Command::new("bash")
        .arg(&script)
        .current_dir(manifest_dir)
        .status()
        .expect("run fetch-go-sdk-examples.sh");
    assert!(status.success(), "fetch-go-sdk-examples.sh failed");
    assert!(marker.exists(), "go-sdk hello example missing after fetch");
}

pub fn ensure_go_build(subject: &SubjectManifest, artifact_name: &str) {
    let artifact_path = subject.source_dir.join(artifact_name);
    if artifact_path.exists() {
        return;
    }

    let Some(build) = &subject.build else {
        return;
    };

    let status = std::process::Command::new(&build.command)
        .args(&build.args)
        .current_dir(&subject.source_dir)
        .status()
        .expect("build go subject");
    assert!(status.success(), "go subject build failed");
    assert!(
        artifact_path.exists(),
        "go build artifact missing: {}",
        artifact_path.display()
    );
}

pub fn ensure_rust_mcp_filesystem_repo(manifest_dir: &Path) {
    let server_dir = manifest_dir.join("external/rust-mcp-filesystem");
    if server_dir.join("Cargo.toml").exists() {
        return;
    }

    std::fs::create_dir_all(manifest_dir.join("external")).expect("create external dir");
    let status = std::process::Command::new("git")
        .args([
            "clone",
            "https://github.com/rust-mcp-stack/rust-mcp-filesystem",
            &server_dir.to_string_lossy(),
        ])
        .status()
        .expect("clone rust-mcp-filesystem");
    assert!(status.success(), "git clone rust-mcp-filesystem failed");
}

pub fn ensure_npm_install(subject: &SubjectManifest) {
    let sdk_marker = subject
        .source_dir
        .join("node_modules/@modelcontextprotocol/sdk/package.json");
    let zod_json_schema = subject
        .source_dir
        .join("node_modules/zod-to-json-schema/dist/esm/index.js");
    let sdk_interfaces = subject.source_dir.join(
        "node_modules/@modelcontextprotocol/sdk/dist/esm/experimental/tasks/interfaces.js",
    );

    if sdk_marker.exists() && zod_json_schema.exists() && sdk_interfaces.exists() {
        return;
    }

    let Some(build) = &subject.build else {
        return;
    };

    let status = std::process::Command::new(&build.command)
        .args(&build.args)
        .current_dir(&subject.source_dir)
        .status()
        .expect("build typescript native MCP npm install");
    assert!(status.success(), "typescript native MCP npm install failed");
    assert!(
        sdk_marker.exists(),
        "missing @modelcontextprotocol/sdk after npm install"
    );
    assert!(
        zod_json_schema.exists(),
        "missing zod-to-json-schema after npm install"
    );
    assert!(
        sdk_interfaces.exists(),
        "incomplete @modelcontextprotocol/sdk install after npm install"
    );
}

pub fn ensure_typescript_sdk_examples(manifest_dir: &Path) {
    let marker = manifest_dir.join("external/typescript-sdk/package.json");
    if marker.exists() {
        return;
    }

    let script = manifest_dir.join("scripts/fetch-typescript-sdk-examples.sh");
    assert!(
        script.exists(),
        "missing fetch script: {}",
        script.display()
    );

    let status = std::process::Command::new("bash")
        .arg(&script)
        .current_dir(manifest_dir)
        .status()
        .expect("run fetch-typescript-sdk-examples.sh");
    assert!(status.success(), "fetch-typescript-sdk-examples.sh failed");
    assert!(marker.exists(), "typescript-sdk missing after fetch");
}


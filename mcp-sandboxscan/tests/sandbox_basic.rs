use std::collections::HashMap;
use std::path::Path;
use std::fs;

use mcp_sandboxscan::sandbox::wasm_runner::WasmRunner;

/// Basic sanity check:
/// a benign MCP tool can be executed under WASI
/// and its runtime outputs are fully captured.
#[test]
fn test_benign_wasm_exec() {
    // define path to compiled WASM tool
    let wasm_path = Path::new("fixtures/benign_tool/tool.wasm");
    let wasm_bytes = fs::read(wasm_path)
        .expect("failed to read benign wasm file");

    let runner = WasmRunner::default();
    let env = HashMap::new();

    let result = runner.run(&wasm_bytes, None, &env, 4096)
        .expect("execution failed");

    assert_eq!(
        result.exit_code, 0,
        "expected normal exit for benign tool"
    );
    assert!(
        result.stdout.contains("PROMPT"),
        "expected PROMPT in stdout"
    );
    assert!(
        result.stderr.is_empty(),
        "expected empty stderr for benign tool"
    );
}


/// Observable abnormal behavior:
/// unauthorized filesystem access should surface
/// as a non-zero exit or WASI trap
#[test]
fn test_fs_violation_observable() {
    let wasm_path = Path::new("fixtures/fs_violation_tool/tool.wasm");
    let wasm_bytes = fs::read(wasm_path)
        .expect("failed to read fs-violation wasm file");

    let runner = WasmRunner::default();
    let env = HashMap::new();

    let result = runner.run(&wasm_bytes, None, &env, 4096)
        .expect("execution failed");

    // not equal
    assert_ne!(
        result.exit_code, 0,
        "expected non-zero exit for fs-violation tool"
    );

    assert!(
        result.stderr.contains("access")
            || result.stderr.contains("trap")
            || result.stderr.contains("permission"),
        "expected WASI fs violation observable in stderr"
    );
}
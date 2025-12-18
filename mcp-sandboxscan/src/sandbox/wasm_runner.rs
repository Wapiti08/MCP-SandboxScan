use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use wasi_common::pipe::{ReadPipe, WritePipe};
use anyhow::Result;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{
    Dir,
    WasiCtx,
    WasiCtxBuilder,
    ResourceTable,
    WasiView,
};

/// Simple WASI host environment that implements WasiView
struct WasiHost {
    table: ResourceTable,
    ctx: WasiCtx,
}

impl WasiView for WasiHost {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

/// Result of a WASM execution (stdout and stderr, exit code)
# [derive(Debug, Clone)]
pub struct WasmExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Minimal WASM sandbox based on wasmtime + WASI
#[derive(Clone)]
pub struct WasmtimeSandbox {
    engine: Engine,
}

impl WasmtimeSandbox {
    /// Create a new WasmtimeSandbox
    pub fn new() -> Self {
        let engine = Engine::default();
        Self { engine }
    }

    /// Run a WASM module with WASI in a sandbox
    /// 
    /// - `wasm_path`: path to .wasm file
    /// - 'args': argv (first arg is usually the program name)
    /// - 'env': environment variables
    pub fn run(
        &self,
        wasm_path: &Path,
        args: &[String],
        env: &HashMap<String, String>,
        max_output_size: usize,
    ) -> Result<WasmExecResult> {
        let wasm_bytes = fs::read(wasm_path)?;
        // create a wasmtime Module from binary, engine is the environment for compilation
        // ? will return Err if compilation fails
        let module = Module::from_binary(&self.engine, &wasm_bytes)?;
        
        // in-memory pipes for stdout/stderr
        let stdout = wasi_common::pipe::WritePipe::new_in_memory();
        let stderr = wasi_common::pipe::WritePipe::new_in_memory();

        // build WASI context
        let mut ctx_builder = WasiCtxBuilder::new();
        
        // program name + args - args.first: check whether at least one arg is present
        if let Some(first) = args.first() {
            ctx_builder = ctx_builder.arg(first)?;
            // add args out of first program name
            for a in args.iter().skip(1) {
                ctx_builder = ctx_builder.arg(a)?;
            }
        }

        // Env vars
        for (k, v) in env {
            ctx_builder = ctx_builder.env(k, v)?;
        }
        
        ctx_builder = ctx_builder
            .stdout(Box::new(stdout.clone()))
            .stderr(Box::new(stderr.clone()));

        let wasi_ctx = ctx_builder.build();

        let mut table = ResourceTable::new();
        let mut host = WasiHost { table, ctx: wasi_ctx };
        // create linker to manage functions inside WebAssembly module
        let mut linker: Linker<WasiHost> = Linker::new(&self.engine);
        // register WASI functions in linker
        wasmtime_wasi::add_to_linker(&mut linker, |h| h)?;
        
        let mut store = Store::new(&self.engine, host);

        // instantiate and call '_start' (the WASI entry point)
        let instance = linker.instantiate(&mut store, &module)?;
        // provide the '_start' function
        let start = instance.get_typed_func::<(), (), _>(&mut store, "_start");

        let exit_code = match start {
            Ok(func) => {
                match func.call(&mut store, ()) {
                    Ok(()) => 0, // normal exit
                    Err(trap) => {
                        // check if it's a WASI exit code
                        if let Some(exit) = trap.i32_exit_status() {
                            exit
                        } else {
                            eprintln!("Trap during execution: {}", trap);
                            -1
                        }
                }
            }
        }
            Err(e) => {
                eprintln!("No _start function or invalid signature: {}", e);
                -1
            }
        };
        // read stdout/stderr from pipes 
        // try_into_inner consumes the WritePipe and returns the inner pipe
        // unwrap will panic if fails
        // into_inner returns the inner buffer as Vec<u8>
        let stdout_bytes = stdout.try_into_inner().unwrap().into_inner();
        let stderr_bytes = stderr.try_into_inner().unwrap().into_inner();

        // truncate outputs if larger than max_output_size
        let stdout_bytes = if stdout_bytes.len() > max_output_size {
            stdout_bytes[..max_output_size].to_vec()
        } else {
            stdout_bytes
        };

        let stderr_bytes = if stderr_bytes.len() > max_output_size {
            stderr_bytes[..max_output_size].to_vec()
        } else {
            stderr_bytes
        };
        
        let duration_ms = start.elapsed().as_millis();

        Ok(WasmExecResult {
            stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
            stderr: String::from_utf8_lossy(&stderr_bytes).to_string(),
            exit_code,
            duration_ms,
        })
    }
}
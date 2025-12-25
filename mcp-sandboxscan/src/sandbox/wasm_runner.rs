use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result};
use wasi_common::pipe::WritePipe;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{
    Dir,
    WasiCtx,
    WasiCtxBuilder,
};

use crate::sandbox::exec_result::WasmExecResult;

pub struct WasmRunner {
    engine: Engine,
}

impl Default for WasmRunner {
    fn default() -> Self {
        Self { engine: Engine::default() }
    }
}

impl WasmRunner {
    pub fn run(
        &self,
        wasm_bytes: &[u8],
        data_dir: Option<&Path>,
        env: &HashMap<String, String>,
        max_output_bytes: usize,
    ) -> Result<WasmExecResult> {
        let start = Instant::now();
        // check whether wasm is legal and check input 
        // illgel instructions or malicious build will be caught here
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .context("failed to create module from wasm bytes")?;

        let stdout = WritePipe::new_in_memory();
        let stderr = WritePipe::new_in_memory();
        
        // wasi and host decoupling - box to heap-allocate a concrete WritePipe
        let mut builder = WasiCtxBuilder::new()
            .stdout(Box::new(stdout.clone()))
            .stderr(Box::new(stderr.clone()));

        // ingest env vars
        for (k, v) in env {
            builder = builder.env(k, v)?;
        }

        // preopen data dir as /data
        if let Some(dir) = data_dir {
            // assign privileged capability to open dir
            let cap_dir = Dir::open_ambient_dir(dir, wasmtime_wasi::ambient_authority())
                .with_context(|| format!("failed to open data dir {}", dir.display()))?;
            builder = builder.preopened_dir(cap_dir, "/data")?;
        }

        let wasi: WasiCtx = builder.build();
        // sandbox status container for running once
        let mut store = Store::new(&self.engine, wasi);
        // wasm imports to host implementations
        let mut linker = Linker::new(&self.engine);

        // add wasi to wasm
        wasmtime_wasi::add_to_linker(&mut linker, |ctx| ctx)?;
        // initialize instance
        let instance = linker.instantiate(&mut store, &module)?;
        // _start is the WASI entrypoint
        let start_func = instance
            .get_typed_func::<(), (), _>(&mut store, "_start")
            .context("failed to get _start function")?;

        let exit_code = match start_func {
            Ok(f) => match f.call(&mut store, ()) {
                Ok(()) => 0, // normal exit
                Err(trap) => {
                    // check if it's a WASI exit code
                    if let Some(exit) = trap.i32_exit_status() {
                        exit
                    } else {
                        eprintln!("WASM trap: {trap}");
                        // other traps
                        -1
                    }
                }
            },
            Err(e) => {
                eprintln!("No _start or invalid signature: {e}");
                -1
            }
        };

        // into_inner returns the inner buffer as Vec<u8>
        let mut stdout_bytes = stdout.try_into_inner().unwrap().into_inner();
        let mut stderr_bytes = stderr.try_into_inner().unwrap().into_inner();

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
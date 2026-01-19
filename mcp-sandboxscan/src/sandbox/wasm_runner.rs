use anyhow::{Context, Result};
use wasmtime::{Engine, Instance, Linker, Module, Store, Error};

use wasmtime_wasi::I32Exit;

use crate::sandbox::exec_result::WasmExecResult;
use crate::sandbox::wasi::WasiRuntime;

// 1) provide wastime enginer (for compile and initialization)
// 2) provide run() -- run wasm bytes and collect result (stdout/stderr/exit_code/duration）
pub struct WasmRunner {
    engine: Engine,
}

impl Default for WasmRunner {
    fn default() -> Self {
        Self {
            // default enginer configuration
            engine: Engine::default(),
        }
    }
}

/// output wastime execution error as exit code
/// - if it is exit(n) from WASI, wasmtime will use I32Exit, extract n
/// - otherwise, it will be trap / crash / violation, use -1
fn error_to_exit_code(err: &Error) -> i32 {
    if let Some(exit) = err.downcast_ref::<I32Exit>() {
        exit.0
    } else {
        -1 // real trap / crash / violation
    }
}

/// use result of _start as exit code
/// - normal => 0
/// - error => corresponding exit code 
fn decode_exit(result: std::result::Result<(), Error>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(e) => error_to_exit_code(&e),
    }
}

impl WasmRunner {
    /// run a wasm module, return collected result
    pub fn run<R: WasiRuntime>(
        &self,
        wasm_bytes: &[u8],
        runtime: &R,
    ) -> Result<WasmExecResult> {
        // 1) compile wasm bytes into Module (Module can be reused)
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .context("failed to compile wasm")?;

        // 2) build wasi context from runtime
        let ctx = runtime.build_ctx()?;

        // 3) store will hold runtime status (ctx, memory, table, globals)
        let mut store = Store::new(&self.engine, ctx);

        // 4) Linker is used to import WASI/host functions, etc., into the linker and connect them to the module.
        let mut linker: Linker<R::Ctx> = Linker::new(&self.engine);

        runtime.add_to_linker(&mut linker)?;

        let instance = linker
            .instantiate(&mut store, &module)
            .context("failed to instantiate wasm")?;

        let exit_code = self.execute::<R>(&mut store, &instance)?;
        // 7) take out collect IO and time counter from runtime
        let io = runtime.take_io()?;

        // 8) ensemble execution results
        Ok(WasmExecResult {
            stdout: String::from_utf8_lossy(&io.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&io.stderr).into_owned(),
            exit_code,
            duration_ms: io.duration_ms,
        })
    }

    /// execute _start in instance and return exit code
    fn execute<R: WasiRuntime>(
        &self,
        store: &mut Store<R::Ctx>,
        instance: &Instance,
    ) -> Result<i32> {
        let start = instance
            .get_typed_func::<(), ()>(&mut *store, "_start")
            .context("missing _start")?;

        let result = start.call(&mut *store, ());
        Ok(decode_exit(result))
    }
}
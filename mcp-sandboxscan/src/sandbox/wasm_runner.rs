use anyhow::{Context, Result};
use wasmtime::{Engine, Instance, Linker, Module, Store, Error};

use wasmtime_wasi::I32Exit;

use crate::sandbox::exec_result::WasmExecResult;
use crate::sandbox::wasi::WasiRuntime;

pub struct WasmRunner {
    engine: Engine,
}

impl Default for WasmRunner {
    fn default() -> Self {
        Self {
            engine: Engine::default(),
        }
    }
}

fn error_to_exit_code(err: &Error) -> i32 {
    if let Some(exit) = err.downcast_ref::<I32Exit>() {
        exit.0
    } else {
        -1 // real trap / crash / violation
    }
}
fn decode_exit(result: std::result::Result<(), Error>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(e) => error_to_exit_code(&e),
    }
}

impl WasmRunner {
    pub fn run<R: WasiRuntime>(
        &self,
        wasm_bytes: &[u8],
        runtime: &R,
    ) -> Result<WasmExecResult> {
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .context("failed to compile wasm")?;

        let ctx = runtime.build_ctx()?;
        let mut store = Store::new(&self.engine, ctx);
        let mut linker: Linker<R::Ctx> = Linker::new(&self.engine);

        runtime.add_to_linker(&mut linker)?;

        let instance = linker
            .instantiate(&mut store, &module)
            .context("failed to instantiate wasm")?;

        let exit_code = self.execute::<R>(&mut store, &instance)?;

        let io = runtime.take_io()?;

        Ok(WasmExecResult {
            stdout: String::from_utf8_lossy(&io.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&io.stderr).into_owned(),
            exit_code,
            duration_ms: io.duration_ms,
        })
    }

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
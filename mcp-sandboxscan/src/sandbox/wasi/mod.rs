pub mod preview1;
use anyhow::Result;
use wasmtime::Linker;
use wasmtime_wasi::WasiView;

/// WASI runtime adapter abstraction.
/// This isolates ABI differences (preview1 / preview2).
pub struct WasiExecutionIO {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub duration_ms: u128,
}

pub trait WasiRuntime {
    type Ctx: wasmtime_wasi::WasiView + Send + 'static;

    fn build_ctx(&self) -> Result<Self::Ctx>;

    fn add_to_linker(&self, linker: &mut Linker<Self::Ctx>) -> Result<()>;

    /// collect execution outputs
    fn take_io(&self) -> Result<WasiExecutionIO>;
}
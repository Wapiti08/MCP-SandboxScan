use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use anyhow::Result;
use wasmtime::Linker;

use wasmtime_wasi::filesystem::{DirPerms, FilePerms};
use wasmtime_wasi::p1::{self, WasiP1Ctx};
use wasmtime_wasi::p2::pipe::MemoryOutputPipe;
use wasmtime_wasi::WasiCtxBuilder;

use super::{WasiExecutionIO, WasiRuntime};

/// Preview1 (_start-based) WASI runtime adapter
pub struct WasiPreview1 {
    pub env: HashMap<String, String>,
    pub data_dir: Option<PathBuf>,
    pub max_output_bytes: usize,

    stdout: MemoryOutputPipe,
    stderr: MemoryOutputPipe,
    start: Mutex<Option<Instant>>,
}

impl WasiPreview1 {
    pub fn new(
        env: HashMap<String, String>,
        data_dir: Option<PathBuf>,
        max_output_bytes: usize,
    ) -> Self {
        Self {
            env,
            data_dir,
            max_output_bytes,
            stdout: MemoryOutputPipe::new(max_output_bytes),
            stderr: MemoryOutputPipe::new(max_output_bytes),
            start: Mutex::new(None),
        }
    }
}

impl WasiRuntime for WasiPreview1 {
    type Ctx = WasiP1Ctx;

    fn build_ctx(&self) -> anyhow::Result<Self::Ctx> {
        *self.start.lock().unwrap() = Some(Instant::now());

        let mut builder = WasiCtxBuilder::new();
        builder.stdout(self.stdout.clone());
        builder.stderr(self.stderr.clone());

        for (k, v) in &self.env {
            builder.env(k, v);
        }
    
        if let Some(dir) = &self.data_dir {
            builder.preopened_dir(
                dir,
                "/data",
                DirPerms::all(),
                FilePerms::all(),
            )?;
        }
    

        Ok(builder.build_p1())
    }

    fn add_to_linker(&self, linker: &mut Linker<Self::Ctx>) -> Result<()> {
        p1::add_to_linker_sync(linker, |ctx: &mut WasiP1Ctx| ctx)?;
        Ok(())
    }

    fn take_io(&self) -> anyhow::Result<WasiExecutionIO> {
        let duration_ms = self
            .start
            .lock()
            .unwrap()
            .take()
            .map(|t| t.elapsed().as_millis())
            .unwrap_or(0);

        let mut stdout = self.stdout.contents().to_vec();
        let mut stderr = self.stderr.contents().to_vec();

        // 双保险截断
        stdout.truncate(self.max_output_bytes);
        stderr.truncate(self.max_output_bytes);

        Ok(WasiExecutionIO {
            stdout,
            stderr,
            duration_ms,
        })
    }
}
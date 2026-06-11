use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use anyhow::Result;
use serde_json::json;
use wasmtime::Linker;

use crate::monitor::event::{MonitorEvent, MonitorEventKind};

use wasmtime_wasi::filesystem::{DirPerms, FilePerms};
use wasmtime_wasi::p1::{self, WasiP1Ctx};
use wasmtime_wasi::p2::pipe::MemoryOutputPipe;
use wasmtime_wasi::WasiCtxBuilder;

use super::{WasiExecutionIO, WasiRuntime};

/// Preview1 (_start-based) WASI runtime adapter
pub struct WasiPreview1 {
    pub env: HashMap<String, String>,
    pub data_dir: Option<PathBuf>,
    pub work_dir: Option<PathBuf>,
    pub guest_root: Option<PathBuf>,
    pub args: Vec<String>,
    pub max_output_bytes: usize,

    stdout: MemoryOutputPipe,
    stderr: MemoryOutputPipe,
    start: Mutex<Option<Instant>>,
    monitor_events: Mutex<Vec<MonitorEvent>>,
}

impl WasiPreview1 {
    pub fn new(
        env: HashMap<String, String>,
        data_dir: Option<PathBuf>,
        max_output_bytes: usize,
    ) -> Self {
        Self::new_with_args(env, data_dir, None, None, Vec::new(), max_output_bytes)
    }

    pub fn new_with_args(
        env: HashMap<String, String>,
        data_dir: Option<PathBuf>,
        work_dir: Option<PathBuf>,
        guest_root: Option<PathBuf>,
        args: Vec<String>,
        max_output_bytes: usize,
    ) -> Self {
        Self {
            env,
            data_dir,
            work_dir,
            guest_root,
            args,
            max_output_bytes,
            stdout: MemoryOutputPipe::new(max_output_bytes),
            stderr: MemoryOutputPipe::new(max_output_bytes),
            start: Mutex::new(None),
            monitor_events: Mutex::new(Vec::new()),
        }
    }

    pub fn take_monitor_events(&self) -> Vec<MonitorEvent> {
        std::mem::take(&mut *self.monitor_events.lock().unwrap())
    }
}

impl WasiRuntime for WasiPreview1 {
    type Ctx = WasiP1Ctx;

    fn build_ctx(&self) -> anyhow::Result<Self::Ctx> {
        *self.start.lock().unwrap() = Some(Instant::now());
        let mut monitor_events = self.monitor_events.lock().unwrap();
        monitor_events.clear();

        let mut builder = WasiCtxBuilder::new();
        builder.stdout(self.stdout.clone());
        monitor_events.push(MonitorEvent {
            kind: MonitorEventKind::CapabilityGranted,
            actor: "wasi-runtime".to_string(),
            target: Some("stdout".to_string()),
            evidence: json!({
                "capability": "stdio",
                "stream": "stdout",
                "max_output_bytes": self.max_output_bytes
            }),
        });

        builder.stderr(self.stderr.clone());
        monitor_events.push(MonitorEvent {
            kind: MonitorEventKind::CapabilityGranted,
            actor: "wasi-runtime".to_string(),
            target: Some("stderr".to_string()),
            evidence: json!({
                "capability": "stdio",
                "stream": "stderr",
                "max_output_bytes": self.max_output_bytes
            }),
        });

        for (k, v) in &self.env {
            builder.env(k, v);
            monitor_events.push(MonitorEvent {
                kind: MonitorEventKind::CapabilityGranted,
                actor: "wasi-runtime".to_string(),
                target: Some(k.clone()),
                evidence: json!({
                    "capability": "env",
                    "key": k,
                    "value_len": v.len()
                }),
            });
        }

        for arg in &self.args {
            builder.arg(arg);
        }

        if let Some(dir) = &self.guest_root {
            builder.preopened_dir(dir, "/", DirPerms::all(), FilePerms::all())?;
            monitor_events.push(MonitorEvent {
                kind: MonitorEventKind::CapabilityGranted,
                actor: "wasi-runtime".to_string(),
                target: Some("/".to_string()),
                evidence: json!({
                    "capability": "filesystem-preopen",
                    "guest_path": "/",
                    "host_path": dir,
                    "dir_perms": "all",
                    "file_perms": "all"
                }),
            });
        }

        if let Some(dir) = &self.work_dir {
            builder.preopened_dir(dir, "/work", DirPerms::all(), FilePerms::all())?;
            monitor_events.push(MonitorEvent {
                kind: MonitorEventKind::CapabilityGranted,
                actor: "wasi-runtime".to_string(),
                target: Some("/work".to_string()),
                evidence: json!({
                    "capability": "filesystem-preopen",
                    "guest_path": "/work",
                    "host_path": dir,
                    "dir_perms": "all",
                    "file_perms": "all"
                }),
            });
        }
    
        if let Some(dir) = &self.data_dir {
            builder.preopened_dir(
                dir,
                "/data",
                DirPerms::all(),
                FilePerms::all(),
            )?;
            monitor_events.push(MonitorEvent {
                kind: MonitorEventKind::CapabilityGranted,
                actor: "wasi-runtime".to_string(),
                target: Some("/data".to_string()),
                evidence: json!({
                    "capability": "filesystem-preopen",
                    "guest_path": "/data",
                    "host_path": dir,
                    "dir_perms": "all",
                    "file_perms": "all"
                }),
            });
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use super::*;

    #[test]
    fn records_wasi_capability_grants() {
        let data_dir = std::env::temp_dir().join(format!(
            "mcp-sandboxscan-wasi-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&data_dir).unwrap();

        let mut env = HashMap::new();
        env.insert("DEMO_SECRET".to_string(), "secret-value".to_string());

        let runtime = WasiPreview1::new(env, Some(data_dir.clone()), 1024);
        runtime.build_ctx().unwrap();

        let events = runtime.take_monitor_events();
        fs::remove_dir_all(&data_dir).unwrap();

        assert!(events.iter().any(|event| {
            event.kind == MonitorEventKind::CapabilityGranted
                && event.target.as_deref() == Some("DEMO_SECRET")
        }));
        assert!(events.iter().any(|event| {
            event.kind == MonitorEventKind::CapabilityGranted
                && event.target.as_deref() == Some("/data")
        }));
        assert!(events.iter().any(|event| {
            event.kind == MonitorEventKind::CapabilityGranted
                && event.target.as_deref() == Some("stdout")
        }));
    }
}
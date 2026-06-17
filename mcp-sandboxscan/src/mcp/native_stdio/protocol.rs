use std::io::{BufRead, Write};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::mcp::transcript::{McpDirection, McpEvent, McpTranscript};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdioFraming {
    Newline,
    ContentLength,
}

pub trait CommandExt {
    fn current_dir_opt(&mut self, dir: Option<&std::path::PathBuf>) -> &mut Self;
}

impl CommandExt for Command {
    fn current_dir_opt(&mut self, dir: Option<&std::path::PathBuf>) -> &mut Self {
        if let Some(dir) = dir {
            self.current_dir(dir)
        } else {
            self
        }
    }
}

pub fn send_message(stdin: &mut impl Write, message: &Value, framing: StdioFraming) -> Result<()> {
    let line = serde_json::to_string(message)?;
    match framing {
        StdioFraming::Newline => {
            stdin.write_all(line.as_bytes())?;
            stdin.write_all(b"\n")?;
        }
        StdioFraming::ContentLength => {
            write!(stdin, "Content-Length: {}\r\n\r\n", line.len())?;
            stdin.write_all(line.as_bytes())?;
        }
    }
    stdin.flush()?;
    Ok(())
}

pub fn read_response_with_id(
    reader: &mut impl BufRead,
    expected_id: u64,
    framing: StdioFraming,
) -> Result<Value> {
    let mut line = String::new();
    loop {
        let value = match framing {
            StdioFraming::Newline => read_newline_json_skip_noise(reader, &mut line)?,
            StdioFraming::ContentLength => read_content_length_json(reader, &mut line)?,
        };

        if value.get("id").and_then(|id| id.as_u64()) == Some(expected_id) {
            return Ok(value);
        }
    }
}

fn read_newline_json_skip_noise(reader: &mut impl BufRead, line: &mut String) -> Result<Value> {
    loop {
        line.clear();
        let n = reader.read_line(line)?;
        if n == 0 {
            bail!("MCP server closed stdout before JSON response");
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str(trimmed) {
            Ok(value) => return Ok(value),
            Err(_) => continue,
        }
    }
}

fn read_content_length_json(reader: &mut impl BufRead, line: &mut String) -> Result<Value> {
    let mut content_length = None;

    loop {
        line.clear();
        let n = reader.read_line(line)?;
        if n == 0 {
            bail!("MCP server closed stdout before headers");
        }

        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }

        if let Some(value) = header.strip_prefix("Content-Length:") {
            content_length = Some(value.trim().parse::<usize>()?);
        }
    }

    let content_length = content_length.context("MCP response missing Content-Length header")?;
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;

    serde_json::from_slice(&body).context("failed to parse MCP Content-Length JSON body")
}

pub fn record(transcript: &mut McpTranscript, direction: McpDirection, payload: &Value) {
    transcript.events.push(McpEvent {
        direction,
        method: payload
            .get("method")
            .and_then(|value| value.as_str())
            .map(|s| s.to_string()),
        payload: payload.clone(),
    });
}

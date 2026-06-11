mod adapter;
mod cli;
mod pipeline;
mod sandbox;
mod scan;
mod subject;
mod taint;
mod study;
mod mcp;
mod monitor;

fn main() -> anyhow::Result<()> {
    cli::main::entry()
}
mod adapter;
mod cli;
mod collect;
mod mcp;
mod monitor;
mod pipeline;
mod sandbox;
mod scan;
mod study;
mod subject;
mod taint;

fn main() -> anyhow::Result<()> {
    cli::main::entry()
}

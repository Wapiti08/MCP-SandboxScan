mod cli;
mod sandbox;
mod scan;
mod taint;

fn main() -> anyhow::Result<()> {
    cli::main::entry()
}
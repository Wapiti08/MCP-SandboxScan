mod driver;
mod protocol;

pub use driver::NativeStdioMcpDriver;
pub use protocol::StdioFraming;

#[cfg(test)]
mod tests;

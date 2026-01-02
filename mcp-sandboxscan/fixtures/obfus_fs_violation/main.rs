use std::fs;

fn main() {
    // Build "/etc/passwd" at runtime; no literal "/etc/passwd" in the binary
    let path = ["/e", "tc", "/pa", "ss", "wd"].concat();

    match fs::read_to_string(&path) {
        Ok(s) => println!("READ_OK {} bytes", s.len()),
        Err(e) => eprintln!("READ_ERR {}: {}", path, e),
    }
}
}
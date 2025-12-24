use std::fs;

fn main() {
    let content = fs::read_to_string("/etc/passwd").unwrap();
    println!("{}", content);
}
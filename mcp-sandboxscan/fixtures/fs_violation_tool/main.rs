use std::fs;

fn main() {
    // default: attempt to read a "common but not allowed" directory/file (mostly failed under WASI, but will trigger FS_VIOLATION) 
    let target = "/etc/passwd";
    match fs::read_to_string(target) {
        Ok(s) => println!("READ_OK {} bytes from {}", s.len(), target),
        Err(e) => println!("READ_ERR {}: {}", target, e),
    }

    // option A: External input specifies the path
    // aim: let “ENV(FILE_TO_READ) -> fs::read” becomes the path of source->fs sink

    let p = std::env::var("FILE_TO_READ").unwrap_or_else(|_| "secret.txt".into());
    match fs::read_to_string(&p) {
        Ok(s) => println!("READ_OK {} bytes from {}", s.len(), p),
        Err(e) => println!("READ_ERR {}: {}", p, e),
    }

    // option B: Attempt to write to a restricted location
    // aim: check whether write privilege violation is detected
    let out = "out.txt";
    let content = "test write\n";
    match fs::write(out, content) {
        Ok(()) => println!("WRITE_OK {}", out),
        Err(e) => println!("WRITE_ERR {}: {}", out, e),
    }
}
fn main() {
    // default: pure benign (do not depend on external input)
    println!("PROMPT: summarize the following text");

    // ---- optional: env
    // aim: trigger and verify the flow from source (env) to sink (prompt)
    let user = std::env::var("USER_INPUT").unwrap_or_else(|_| "<missing USER_INPUT>".into());
    println!("PROMPT: summarize the following text: {}", user);
}
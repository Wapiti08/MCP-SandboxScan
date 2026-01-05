fn main() {
    // Does NOT read DEMO_SECRET at all.
    // But outputs text that may coincidentally match a short env token.
    println!(
        "{{\"result\":{{\"msg\":\"This is a harmless string with abcd inside: xxabcdyyzz\"}}}}"
    );

    // Add more leaf strings to mimic realistic tool output noise
    println!(
        "{{\"result\":{{\"detail\":\"execution completed successfully; no secrets were accessed.\"}}}}"
    );
}
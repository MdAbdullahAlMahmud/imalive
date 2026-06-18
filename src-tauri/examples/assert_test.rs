//! Standalone check that the keep-awake mechanism creates a real OS assertion.
//! Run: `cargo run --example assert_test`. Holds for 8s, then releases.

fn main() {
    println!("creating assertion...");
    let _guard = keepawake::Builder::default()
        .display(true)
        .idle(true)
        .reason("ImAlive assert_test")
        .app_name("ImAlive")
        .app_reverse_domain("com.mkrlabs.imalive")
        .create()
        .expect("failed to create keep-awake assertion");

    println!("HELD: assertion active for 8 seconds");
    std::thread::sleep(std::time::Duration::from_secs(8));
    println!("releasing (guard dropped)");
}

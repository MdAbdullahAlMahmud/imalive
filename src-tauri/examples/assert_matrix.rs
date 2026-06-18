//! Parametrized keep-awake check: `cargo run --example assert_matrix -- <display> <idle>`
//! e.g. `assert_matrix true false`. Holds for 6s so a `pmset` snapshot can
//! confirm exactly which assertions are created for each option combination.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let display = args.get(1).map(|s| s == "true").unwrap_or(true);
    let idle = args.get(2).map(|s| s == "true").unwrap_or(true);

    println!("requesting display={display} idle={idle}");
    let _guard = keepawake::Builder::default()
        .display(display)
        .idle(idle)
        .reason("ImAlive assert_matrix")
        .app_name("ImAlive")
        .app_reverse_domain("com.mkrlabs.imalive")
        .create()
        .expect("failed to create keep-awake assertion");

    println!("HELD");
    std::thread::sleep(std::time::Duration::from_secs(6));
    println!("released");
}

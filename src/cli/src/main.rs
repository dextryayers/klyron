fn main() {
    klyron_cli::run_cli().unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });
}

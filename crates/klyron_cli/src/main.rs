fn main() -> Result<(), Box<dyn std::error::Error>> {
    klyron_cli::run_cli().map_err(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    })
}

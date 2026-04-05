fn main() {
    if let Err(e) = sure_unpack::pack::run::run() {
        eprintln!("pack: {e}");
        std::process::exit(e.exit_code());
    }
}

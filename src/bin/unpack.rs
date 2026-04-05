fn main() {
    if let Err(e) = sure_unpack::unpack::run::run() {
        eprintln!("unpack: {e}");
        std::process::exit(e.exit_code());
    }
}

fn main() {
    if let Err(err) = treefold::app::run() {
        eprintln!("treefold error: {err}");
        std::process::exit(1);
    }
}

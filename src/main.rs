fn main() {
    std::process::exit(pymermaider_wasm::cli::run(
        std::env::args().skip(1).collect(),
    ));
}

mod menu;
mod operations;
mod storage;
mod validation;

fn main() {
    if let Err(e) = menu::run_menu() {
        eprintln!("Erro: {}", e);
        std::process::exit(1);
    }
}

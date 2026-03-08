use std::io::{self, Write};
use super::operations::{
    create_inflation_entry, list_inflation_entries, edit_inflation_entry, delete_inflation_entry,
};

pub fn run_menu() -> io::Result<()> {
    loop {
        println!("\n╔══════════════════════════════════════╗");
        println!("║   Gerenciador de Taxas de Inflação   ║");
        println!("╚══════════════════════════════════════╝");
        println!("1. Criar nova taxa");
        println!("2. Listar taxas");
        println!("3. Editar taxa");
        println!("4. Deletar taxa");
        println!("Q. Sair");
        println!("────────────────────────────────────────");

        print!("Escolha uma opção: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim() {
            "1" => {
                create_inflation_entry()?;
                prompt_continue()?;
            }
            "2" => {
                list_inflation_entries()?;
                prompt_continue()?;
            }
            "3" => {
                edit_inflation_entry()?;
                prompt_continue()?;
            }
            "4" => {
                delete_inflation_entry()?;
                prompt_continue()?;
            }
            "Q" | "q" => {
                println!("\nAté logo!");
                break;
            }
            _ => {
                println!("✗ Opção inválida. Tente novamente.");
            }
        }
    }

    Ok(())
}

pub fn prompt_continue() -> io::Result<()> {
    print!("\nDeseja fazer outra operação? (S/N): ");
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if !response.trim().eq_ignore_ascii_case("s") {
        // User wants to exit
        println!("Até logo!");
        std::process::exit(0);
    }

    Ok(())
}

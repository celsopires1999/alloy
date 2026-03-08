use super::storage::{load_inflation_data, save_inflation_data};
use super::validation::{
    validate_rate, validate_year, validate_year_is_unique, validate_years_ascending,
};
use alloy::inflation::AnnualInflationEntry;
use std::io::{self, Write};

pub fn create_inflation_entry() -> io::Result<()> {
    println!("\n--- Criar Nova Taxa de Inflação ---");

    let year = loop {
        print!("Digite o ano: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match validate_year(&input) {
            Ok(y) => break y,
            Err(e) => {
                println!("✗ {}", e);
                continue;
            }
        }
    };

    let rate_str = loop {
        print!("Digite a taxa de inflação (até 2 casas decimais): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match validate_rate(input.trim()) {
            Ok(_) => break input.trim().to_string(),
            Err(e) => {
                println!("✗ {}", e);
                continue;
            }
        }
    };

    // Load current data and check for duplicates
    let mut data = load_inflation_data()?;
    let existing_years: Vec<u32> = data.0.iter().map(|e| e.get_year()).collect();

    if let Err(e) = validate_year_is_unique(year, &existing_years) {
        println!("✗ {}", e);
        return Ok(());
    }

    // Create new entry using builder
    let new_entry = match AnnualInflationEntry::builder()
        .with_year(year)
        .with_inflation(&rate_str)
        .build()
    {
        Ok(entry) => entry,
        Err(e) => {
            println!("✗ Erro ao criar entrada: {}", e);
            return Ok(());
        }
    };

    // Add new entry
    data.0.push(new_entry);

    // Sort by year for validation
    data.0.sort_by_key(|e| e.get_year());

    // Validate ascending order
    let years: Vec<u32> = data.0.iter().map(|e| e.get_year()).collect();
    if let Err(e) = validate_years_ascending(&years) {
        println!("✗ Erro de validação: {}", e);
        return Ok(());
    }

    // Save
    if let Err(e) = save_inflation_data(&data) {
        println!("✗ Erro ao salvar: {}", e);
        return Ok(());
    }

    println!("✓ Taxa de inflação criada com sucesso!");
    Ok(())
}

pub fn list_inflation_entries() -> io::Result<()> {
    println!("\n--- Taxas de Inflação ---");

    let data = load_inflation_data()?;

    if data.0.is_empty() {
        println!("Nenhuma taxa de inflação registrada.");
        return Ok(());
    }

    // Print table header
    println!("{:<10} {:<15}", "Ano", "Taxa (%)");
    println!("{}", "-".repeat(25));

    for entry in &data.0 {
        println!("{:<10} {:<15}", entry.get_year(), entry.get_inflation());
    }

    Ok(())
}

pub fn edit_inflation_entry() -> io::Result<()> {
    println!("\n--- Editar Taxa de Inflação ---");

    let data = load_inflation_data()?;

    if data.0.is_empty() {
        println!("Nenhuma taxa de inflação registrada.");
        return Ok(());
    }

    // Show list
    println!("Taxas existentes:");
    println!("{:<10} {:<15}", "Ano", "Taxa (%)");
    println!("{}", "-".repeat(25));
    for entry in &data.0 {
        println!("{:<10} {:<15}", entry.get_year(), entry.get_inflation());
    }

    let year_to_edit = loop {
        print!("\nDigite o ano a editar: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match validate_year(&input) {
            Ok(y) => {
                if data.0.iter().any(|e| e.get_year() == y) {
                    break y;
                } else {
                    println!("✗ Ano não encontrado");
                }
            }
            Err(e) => {
                println!("✗ {}", e);
            }
        }
    };

    let new_rate_str = loop {
        print!("Digite a nova taxa (até 2 casas decimais): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match validate_rate(input.trim()) {
            Ok(_) => break input.trim().to_string(),
            Err(e) => {
                println!("✗ {}", e);
                continue;
            }
        }
    };

    // Find and update entry
    let mut found = false;
    let mut new_data = Vec::new();

    for entry in data.0 {
        if entry.get_year() == year_to_edit {
            let updated_entry = match AnnualInflationEntry::builder()
                .with_year(year_to_edit)
                .with_inflation(&new_rate_str)
                .build()
            {
                Ok(e) => e,
                Err(err) => {
                    println!("✗ Erro ao atualizar: {}", err);
                    return Ok(());
                }
            };
            new_data.push(updated_entry);
            found = true;
        } else {
            new_data.push(entry);
        }
    }

    if !found {
        println!("✗ Ano não encontrado");
        return Ok(());
    }

    let updated_data = super::storage::InflationDataFile(new_data);

    // Validate ascending order
    let years: Vec<u32> = updated_data.0.iter().map(|e| e.get_year()).collect();
    if let Err(e) = validate_years_ascending(&years) {
        println!("✗ Erro de validação: {}", e);
        return Ok(());
    }

    // Save
    if let Err(e) = save_inflation_data(&updated_data) {
        println!("✗ Erro ao salvar: {}", e);
        return Ok(());
    }

    println!("✓ Taxa atualizada com sucesso!");
    Ok(())
}

pub fn delete_inflation_entry() -> io::Result<()> {
    println!("\n--- Deletar Taxa de Inflação ---");

    let data = load_inflation_data()?;

    if data.0.is_empty() {
        println!("Nenhuma taxa de inflação registrada.");
        return Ok(());
    }

    // Show list
    println!("Taxas existentes:");
    println!("{:<10} {:<15}", "Ano", "Taxa (%)");
    println!("{}", "-".repeat(25));
    for entry in &data.0 {
        println!("{:<10} {:<15}", entry.get_year(), entry.get_inflation());
    }

    let year_to_delete = loop {
        print!("\nDigite o ano a deletar: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match validate_year(&input) {
            Ok(y) => {
                if data.0.iter().any(|e| e.get_year() == y) {
                    break y;
                } else {
                    println!("✗ Ano não encontrado");
                }
            }
            Err(e) => {
                println!("✗ {}", e);
            }
        }
    };

    // Confirmation
    print!(
        "Tem certeza que deseja deletar o ano {}? (S/N): ",
        year_to_delete
    );
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if !response.trim().eq_ignore_ascii_case("s") {
        println!("Operação cancelada.");
        return Ok(());
    }

    // Delete
    let mut updated_data = data;
    updated_data.0.retain(|e| e.get_year() != year_to_delete);

    // Save
    if let Err(e) = save_inflation_data(&updated_data) {
        println!("✗ Erro ao salvar: {}", e);
        return Ok(());
    }

    println!("✓ Taxa deletada com sucesso!");
    Ok(())
}

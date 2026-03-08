use alloy::inflation::AnnualInflationEntry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const DATA_DIR: &str = "data/inflation";
const RATES_FILE: &str = "data/inflation/rates.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InflationDataFile(pub Vec<AnnualInflationEntry>);

impl InflationDataFile {
    pub fn new() -> Self {
        InflationDataFile(Vec::new())
    }
}

pub fn ensure_data_dir() -> std::io::Result<()> {
    if !Path::new(DATA_DIR).exists() {
        fs::create_dir_all(DATA_DIR)?;
    }
    Ok(())
}

pub fn load_inflation_data() -> std::io::Result<InflationDataFile> {
    ensure_data_dir()?;

    if !Path::new(RATES_FILE).exists() {
        // Create empty file with empty array
        let empty_data = InflationDataFile::new();
        save_inflation_data(&empty_data)?;
        return Ok(empty_data);
    }

    let content = fs::read_to_string(RATES_FILE)?;
    match serde_json::from_str::<InflationDataFile>(&content) {
        Ok(data) => Ok(data),
        Err(e) => {
            eprintln!("✗ Erro ao ler arquivo JSON: {}", e);
            Ok(InflationDataFile::new())
        }
    }
}

pub fn save_inflation_data(data: &InflationDataFile) -> std::io::Result<()> {
    ensure_data_dir()?;

    let json_string = match serde_json::to_string_pretty(&data.0) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("✗ Erro ao serializar dados: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Serialization error: {}", e),
            ));
        }
    };

    fs::write(RATES_FILE, json_string)?;
    Ok(())
}

// cleaning.rs
// This module handles data loading and cleaning for state-level fuel and generation statistics from the EIA-923 dataset.

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use csv::ReaderBuilder;
use serde::Deserialize;

/// Struct representing a deserialized row from the CSV file.
/// Fields are mapped to exact CSV column headers.
#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(rename = "Plant State")]
    pub state: String,

    #[serde(rename = "Total Fuel Consumption\nMMBtu")]
    pub fuel: String,

    #[serde(rename = "Net Generation\n(Megawatthours)")]
    pub r#gen: String,
}

/// Aggregated totals for each state.
#[derive(Debug, Default)]
pub struct StateStats {
    pub total_fuel: f64,
    pub total_gen: f64,
}

/// Reads and cleans a CSV file, returning a HashMap of state statistics.
/// 
/// # Arguments
/// * `file_path` - The path to the input CSV file
///
/// # Returns
/// * `HashMap<String, StateStats>` where the key is the state code
pub fn load_state_efficiency(file_path: &str) -> Result<HashMap<String, StateStats>, Box<dyn Error>> {
    println!("Attempting to open file: {}", file_path);

    let file = File::open(file_path)?;
    let mut lines = BufReader::new(file).lines();

    // Skip metadata header lines (non-CSV rows)
    for _ in 0..5 {
        lines.next();
    }

    // Join remaining lines to form valid CSV content
    let csv_data: Vec<u8> = lines
        .filter_map(Result::ok)
        .collect::<Vec<String>>()
        .join("\n")
        .into_bytes();

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_data.as_slice());

    let headers = rdr.headers()?.clone();
    println!("🟢 Actual headers: {:?}", headers);

    let mut state_map: HashMap<String, StateStats> = HashMap::new();
    let mut valid_rows = 0;
    let mut skipped_rows = 0;

    for result in rdr.deserialize::<Record>() {
        let record = match result {
            Ok(r) => r,
            Err(_) => {
                skipped_rows += 1;
                continue;
            }
        };

        // Parse and clean fuel and generation values
        let fuel_val: f64 = match record.fuel.replace(",", "").parse() {
            Ok(v) => v,
            Err(_) => {
                skipped_rows += 1;
                continue;
            }
        };

        let gen_val: f64 = match record.r#gen.replace(",", "").parse() {
            Ok(v) => v,
            Err(_) => {
                skipped_rows += 1;
                continue;
            }
        };

        if gen_val == 0.0 {
            skipped_rows += 1;
            continue;
        }

        // Accumulate data by state
        let entry = state_map.entry(record.state.clone()).or_default();
        entry.total_fuel += fuel_val;
        entry.total_gen += gen_val;
        valid_rows += 1;
    }

    println!("✅ Parsed: {} valid rows | ❌ Skipped: {} rows", valid_rows, skipped_rows);
    Ok(state_map)
}


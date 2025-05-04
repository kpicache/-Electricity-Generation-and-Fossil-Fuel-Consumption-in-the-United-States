// Main.rs
// This is the main program logic for computing fossil fuel efficiency change between 2019 and 2020 across U.S. states using EIA-923 data.

mod cleaning;

use cleaning::{load_state_efficiency, StateStats};
use std::collections::HashMap;
use std::error::Error;
use csv::WriterBuilder;

/// Struct to hold the year-over-year efficiency data for a state.
#[derive(Debug)]
struct StateEfficiency {
    /// State abbreviation (e.g., "CA", "TX").
    state: String,

    /// Efficiency in 2019 (fuel used per MWh).
    eff_2019: f64,

    /// Efficiency in 2020.
    eff_2020: f64,

    /// Change in efficiency (2020 - 2019).
    delta: f64,

    /// Absolute change in efficiency (magnitude only).
    abs_delta: f64,
}

/// Computes efficiency change metrics per state based on aggregated data.
/// # Arguments
/// * `stats_2019` - Map of 2019 state data
/// * `stats_2020` - Map of 2020 state data
/// # Returns
/// * `Vec<StateEfficiency>` representing efficiency differences by state
fn compute_efficiency_changes(
    stats_2019: &HashMap<String, StateStats>,
    stats_2020: &HashMap<String, StateStats>,
) -> Vec<StateEfficiency> {
    let mut output = Vec::new();

    for (state, stat_2019) in stats_2019 {
        if let Some(stat_2020) = stats_2020.get(state) {
            if stat_2019.total_gen == 0.0 || stat_2020.total_gen == 0.0 {
                continue;
            }

            // Calculate efficiency = fuel / generation
            let eff_2019 = stat_2019.total_fuel / stat_2019.total_gen;
            let eff_2020 = stat_2020.total_fuel / stat_2020.total_gen;
            let delta = eff_2020 - eff_2019;
            let abs_delta = delta.abs();

            output.push(StateEfficiency {
                state: state.clone(),
                eff_2019,
                eff_2020,
                delta,
                abs_delta,
            });
        }
    }

    output
}

/// Displays top N states with the largest changes in efficiency.
fn display_top_states(data: &[StateEfficiency], top_n: usize) {
    println!(
        "{:<10} {:>15} {:>15} {:>15} {:>15}",
        "State", "Eff_2019", "Eff_2020", "Change", "Abs Change"
    );
    println!("{}", "-".repeat(75));

    for item in data.iter().take(top_n) {
        println!(
            "{:<10} {:>15.3} {:>15.3} {:>15.3} {:>15.3}",
            item.state, item.eff_2019, item.eff_2020, item.delta, item.abs_delta
        );
    }
}

/// Writes the computed efficiency change data to a CSV output file.
fn write_efficiency_csv(path: &str, data: &[StateEfficiency]) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new().from_path(path)?;
    wtr.write_record(&[
        "State", "Efficiency_2019", "Efficiency_2020", "Delta_Efficiency", "Abs_Change",
    ])?;

    for item in data {
        wtr.write_record(&[
            &item.state,
            &format!("{:.6}", item.eff_2019),
            &format!("{:.6}", item.eff_2020),
            &format!("{:.6}", item.delta),
            &format!("{:.6}", item.abs_delta),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

/// Main program entry point:
/// - Loads the 2019 and 2020 CSVs
/// - Computes fossil fuel efficiency per state
/// - Outputs top movers and saves results to CSV
fn main() -> Result<(), Box<dyn Error>> {
    println!("Running from: {}", std::env::current_dir()?.display());

    let file_2019 = "../data_csv_files/2019.csv";
    let file_2020 = "../data_csv_files/2020.csv";

    println!("Loading 2019 data...");
    let stats_2019 = load_state_efficiency(file_2019)?;

    println!("Loading 2020 data...");
    let stats_2020 = load_state_efficiency(file_2020)?;

    println!("Computing efficiency changes...");
    let mut changes = compute_efficiency_changes(&stats_2019, &stats_2020);
    changes.sort_by(|a, b| b.abs_delta.partial_cmp(&a.abs_delta).unwrap());

    println!("\nTop 10 States by Change in Fossil Fuel Efficiency:\n");
    display_top_states(&changes, 10);

    println!("\nSaving full results to 'efficiency_changes.csv'...");
    write_efficiency_csv("efficiency_changes.csv", &changes)?;

    println!("Done.");
    Ok(())
}

// Cargo Tests
#[cfg(test)]
mod tests {
   use super::*;


   #[test]
   fn test_efficiency_computation() {
       let mut stats_2019 = HashMap::new();
       let mut stats_2020 = HashMap::new();


       stats_2019.insert(
           "TX".to_string(),
           StateStats {
               total_fuel: 1000.0,
               total_gen: 100.0,
           },
       );
       stats_2020.insert(
           "TX".to_string(),
           StateStats {
               total_fuel: 800.0,
               total_gen: 100.0,
           },
       );


       let results = compute_efficiency_changes(&stats_2019, &stats_2020);
       assert_eq!(results.len(), 1);
       let tx = &results[0];
       assert_eq!(tx.state, "TX");
       assert!((tx.eff_2019 - 10.0).abs() < 1e-6);
       assert!((tx.eff_2020 - 8.0).abs() < 1e-6);
       assert!((tx.delta + 2.0).abs() < 1e-6);
       assert!((tx.abs_delta - 2.0).abs() < 1e-6);
   }


   #[test]
   fn test_skipping_zero_generation() {
       let mut stats_2019 = HashMap::new();
       let mut stats_2020 = HashMap::new();


       stats_2019.insert(
           "CA".to_string(),
           StateStats {
               total_fuel: 500.0,
               total_gen: 0.0,
           },
       );
       stats_2020.insert(
           "CA".to_string(),
           StateStats {
               total_fuel: 900.0,
               total_gen: 0.0,
           },
       );


       let results = compute_efficiency_changes(&stats_2019, &stats_2020);
       assert_eq!(results.len(), 0);
   }
}

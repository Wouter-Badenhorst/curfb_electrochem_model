use std::error::Error;
use csv::{Reader, Writer};
use std::collections::HashMap;

pub fn process_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path(input_path)?;
    let mut data: Vec<(i64, f64, f64)> = Vec::new();  // (time, voltage, current)
    
    // Get the first timestamp to calculate relative time
    let mut first_timestamp: Option<i64> = None;

    // Process each record
    for result in rdr.records() {
        let record = result?;
        
        // Parse values with proper error handling
        let timestamp: i64 = record[0].trim().parse()?;
        let voltage: f64 = record[1].trim().parse()?;
        let is_positive: bool = record[2].trim().to_uppercase() == "TRUE";
        let current: f64 = record[3].trim().parse()?;

        // Set first timestamp if not set
        if first_timestamp.is_none() {
            first_timestamp = Some(timestamp);
        }

        // Calculate relative time in seconds
        let relative_time = timestamp - first_timestamp.unwrap();

        // Make current negative if direction is false
        let adjusted_current = if is_positive { current } else { -current };

        data.push((relative_time, voltage, adjusted_current));
    }

    // Bin data into 60-second intervals
    let mut binned_data: HashMap<i64, Vec<(f64, f64)>> = HashMap::new();
    
    for (time, voltage, current) in data {
        let bin = time / 60; // 60-second bins
        binned_data.entry(bin).or_insert_with(Vec::new).push((voltage, current));
    }

    // Calculate averages for each bin and sort by time
    let mut final_data: Vec<(i64, f64, f64)> = binned_data
        .iter()
        .map(|(&bin, values)| {
            let avg_voltage = values.iter().map(|(v, _)| v).sum::<f64>() / values.len() as f64;
            let avg_current = values.iter().map(|(_, c)| c).sum::<f64>() / values.len() as f64;
            (bin * 60, avg_voltage, avg_current)
        })
        .collect();

    final_data.sort_by_key(|k| k.0);

    // Write to output CSV
    let mut wtr = Writer::from_path(output_path)?;
    wtr.write_record(&["Time (s)", "Voltage", "Current"])?;

    for (time, voltage, current) in final_data {
        wtr.write_record(&[
            time.to_string(),
            voltage.to_string(),
            current.to_string()
        ])?;
    }

    wtr.flush()?;
    Ok(())
}
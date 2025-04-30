use plotters::prelude::*;
use std::error::Error;
use csv::Reader;

fn read_voltage_data(file_path: &str) -> Result<Vec<(f32, f32)>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(file_path)?;
    let mut data = Vec::new();
    
    // Assuming time is in column 0 and voltage in column 1
    for result in rdr.records() {
        let record = result?;
        let time: f32 = record[0].parse()?;
        let voltage: f32 = record[1].parse()?;
        data.push((time, voltage));
    }
    Ok(data)
}

pub fn plot_voltage_comparison() -> Result<(), Box<dyn Error>> {
    // Read data from both files
    let real_data = read_voltage_data("data.csv")?;
    let simulated_data = read_voltage_data("output.csv")?;

    // Create the plot
    let root = BitMapBackend::new("voltage_comparison.png", (1200, 800))
        .into_drawing_area();
    root.fill(&WHITE)?;

    // Calculate data ranges dynamically
    let time_min = real_data.iter()
        .chain(simulated_data.iter())
        .map(|(t, _)| *t)
        .reduce(f32::min)
        .unwrap_or(0.0);
    let time_max = real_data.iter()
        .chain(simulated_data.iter())
        .map(|(t, _)| *t)
        .reduce(f32::max)
        .unwrap_or(200.0);
    
    let voltage_min = real_data.iter()
        .chain(simulated_data.iter())
        .map(|(_, v)| *v)
        .reduce(f32::min)
        .unwrap_or(-1.0);
    let voltage_max = real_data.iter()
        .chain(simulated_data.iter())
        .map(|(_, v)| *v)
        .reduce(f32::max)
        .unwrap_or(1.0);

    let mut chart = ChartBuilder::on(&root)
        .caption("Voltage vs Time Comparison", ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(time_min..time_max, voltage_min..voltage_max)?;

    // Configure the chart
    chart.configure_mesh()
        .x_desc("Time (s)")
        .y_desc("Voltage (V)")
        .draw()?;

    // Plot real data with owned values
    chart.draw_series(LineSeries::new(
        real_data.into_iter().map(|(x, y)| (x, y)),
        &BLUE,
    ))?.label("Experimental Data")
      .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // Plot simulated data with owned values
    chart.draw_series(LineSeries::new(
        simulated_data.into_iter().map(|(x, y)| (x, y)),
        &RED,
    ))?.label("Model Output")
      .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    // Add legend
    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .position(SeriesLabelPosition::UpperRight)
        .draw()?;

    Ok(())
}

// Optional test function
#[cfg(test)]
fn main() -> Result<(), Box<dyn Error>> {
    plot_voltage_comparison()?;
    println!("Plot saved as voltage_comparison.png");
    Ok(())
}
use eframe::egui;
use egui::ViewportBuilder;
use egui_plot::{Line, Plot, PlotPoints};
use std::error::Error;
use csv::Reader;

struct PlotViewer {
    // Model data
    time_data: Vec<f64>,
    voltage_data: Vec<f64>,
    c1a_data: Vec<f64>,
    c2a_data: Vec<f64>,
    c1c_data: Vec<f64>,
    c0c_data: Vec<f64>,
    // Experimental data
    exp_time: Vec<f64>,
    exp_voltage: Vec<f64>,
    // Display toggles
    show_voltage: bool,
    show_concentration: bool,
    show_experimental: bool,
}

impl PlotViewer {
    fn new() -> Result<Self, Box<dyn Error>> {
        println!("Reading model data from output.csv...");
        let (time, voltage, c1c, c0c, c1a, c2a) = Self::read_model_data("output.csv")?;

        println!("Reading experimental data from data.csv...");
        let (exp_time, exp_voltage) = Self::read_experimental_data("data.csv")?;

        Ok(Self {
            time_data: time,
            voltage_data: voltage,
            c1a_data: c1a,
            c2a_data: c2a,
            c1c_data: c1c,
            c0c_data: c0c,
            exp_time,
            exp_voltage,
            show_voltage: true,
            show_concentration: true,
            show_experimental: true,
        })
    }

    fn read_model_data(file_path: &str) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), Box<dyn Error>> {
        let mut rdr = Reader::from_path(file_path)?;
        let mut time = Vec::new();
        let mut voltage = Vec::new();
        let mut c1c = Vec::new();
        let mut c0c = Vec::new();
        let mut c1a = Vec::new();
        let mut c2a = Vec::new();
        
        // Skip header row
        let headers = rdr.headers()?.clone();
        println!("CSV headers: {:?}", headers);
        
        for (i, result) in rdr.records().enumerate() {
            let record = result?;
            
            time.push(record[0].trim().parse()?);
            voltage.push(record[1].trim().parse()?);
            c1c.push(record[2].trim().parse()?);
            c0c.push(record[3].trim().parse()?);
            c1a.push(record[4].trim().parse()?);
            c2a.push(record[5].trim().parse()?);
        }

        println!("Successfully read {} data points", time.len());
        Ok((time, voltage, c1c, c0c, c1a, c2a))
    }

    fn read_experimental_data(file_path: &str) -> Result<(Vec<f64>, Vec<f64>), Box<dyn Error>> {
        let mut rdr = Reader::from_path(file_path)?;
        let mut time = Vec::new();
        let mut voltage = Vec::new();
        
        for result in rdr.records() {
            let record = result?;
            time.push(record[0].trim().parse()?);
            voltage.push(record[1].trim().parse()?);
        }

        println!("Successfully read {} experimental data points", time.len());
        Ok((time, voltage))
    }

    fn refresh_data(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Refreshing data...");
        let (time, voltage, c1c, c0c, c1a, c2a) = Self::read_model_data("output.csv")?;
        let (exp_time, exp_voltage) = Self::read_experimental_data("data.csv")?;

        self.time_data = time;
        self.voltage_data = voltage;
        self.c1a_data = c1a;
        self.c2a_data = c2a;
        self.c1c_data = c1c;
        self.c0c_data = c0c;
        self.exp_time = exp_time;
        self.exp_voltage = exp_voltage;

        println!("Data refresh complete");
        Ok(())
    }
}

impl eframe::App for PlotViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_voltage, "Voltage");
                ui.checkbox(&mut self.show_concentration, "Concentration");
                ui.checkbox(&mut self.show_experimental, "Experimental Data");
                
                // Add refresh button
                if ui.button("🔄 Refresh Data").clicked() {
                    if let Err(e) = self.refresh_data() {
                        eprintln!("Error refreshing data: {}", e);
                    }
                }
            });

            let available_height = ui.available_height();
            let plot_height = (available_height - 40.0) / 2.0; // Account for padding

            ui.vertical(|ui| {
                // Voltage plot
                if self.show_voltage {
                    ui.group(|ui| {
                        Plot::new("voltage_plot")
                            .height(plot_height)
                            .width(ui.available_width())  // Make plot fill available width
                            .legend(egui_plot::Legend::default())  // Add legend
                            .auto_bounds_x()  // Automatically fit x bounds
                            .auto_bounds_y()  // Automatically fit y bounds
                            .show(ui, |plot_ui| {
                                // Model voltage
                                let voltage_points: Vec<[f64; 2]> = self.time_data.iter()
                                    .zip(&self.voltage_data)
                                    .map(|(&t, &v)| [t, v])
                                    .collect();
                                
                                plot_ui.line(Line::new(PlotPoints::new(voltage_points))
                                    .name("Model")
                                    .width(2.0)
                                    .color(egui::Color32::BLUE));

                                // Experimental voltage
                                if self.show_experimental {
                                    let exp_points: Vec<[f64; 2]> = self.exp_time.iter()
                                        .zip(&self.exp_voltage)
                                        .map(|(&t, &v)| [t, v])
                                        .collect();
                                    
                                    plot_ui.line(Line::new(PlotPoints::new(exp_points))
                                        .name("Experimental")
                                        .width(2.0)
                                        .color(egui::Color32::RED));
                                }
                            });
                    });
                }

                // Concentration plot
                if self.show_concentration {
                    ui.group(|ui| {
                        Plot::new("concentration_plot")
                            .height(plot_height)
                            .width(ui.available_width())  // Make plot fill available width
                            .legend(egui_plot::Legend::default())  // Add legend
                            .auto_bounds_x()  // Automatically fit x bounds
                            .auto_bounds_y()  // Automatically fit y bounds
                            .show(ui, |plot_ui| {
                                // C1a concentration
                                let c1a_points: Vec<[f64; 2]> = self.time_data.iter()
                                    .zip(&self.c1a_data)
                                    .map(|(&t, &c)| [t, c])
                                    .collect();
                                plot_ui.line(Line::new(PlotPoints::new(c1a_points))
                                    .name("C1a")
                                    .width(2.0)
                                    .color(egui::Color32::RED));

                                // C2a concentration
                                let c2a_points: Vec<[f64; 2]> = self.time_data.iter()
                                    .zip(&self.c2a_data)
                                    .map(|(&t, &c)| [t, c])
                                    .collect();
                                plot_ui.line(Line::new(PlotPoints::new(c2a_points))
                                    .name("C2a")
                                    .width(2.0)
                                    .color(egui::Color32::GREEN));

                                // C1c concentration
                                let c1c_points: Vec<[f64; 2]> = self.time_data.iter()
                                    .zip(&self.c1c_data)
                                    .map(|(&t, &c)| [t, c])
                                    .collect();
                                plot_ui.line(Line::new(PlotPoints::new(c1c_points))
                                    .name("C1c")
                                    .width(2.0)
                                    .color(egui::Color32::YELLOW));

                                // C0c concentration
                                let c0c_points: Vec<[f64; 2]> = self.time_data.iter()
                                    .zip(&self.c0c_data)
                                    .map(|(&t, &c)| [t, c])
                                    .collect();
                                plot_ui.line(Line::new(PlotPoints::new(c0c_points))
                                    .name("C0c")
                                    .width(2.0)
                                    .color(egui::Color32::LIGHT_BLUE));
                            });
                    });
                }
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    match PlotViewer::new() {
        Ok(app) => eframe::run_native(
            "Electrochemical Model Viewer",
            options,
            Box::new(|_cc| Box::new(app))
        ),
        Err(e) => {
            eprintln!("Error initializing plot viewer: {}", e);
            Ok(())
        }
    }
}
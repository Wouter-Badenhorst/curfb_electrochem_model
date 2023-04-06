use std::io::Write;                                                                                                                                                                                                                                                                                                                           
use std::fs::File; 
use std::io::BufWriter;
use std::error::Error;
use csv::Reader;

const FARADAY_CONSTANT: f32 = 96485.0;
const ELECTROLYTE_VOLUME: f32 = 3e-6; 
const FORMAL_POTENTIAL: f32 = 0.65;
const GAS_CONSTANT: f32 = 8.3145;
const COPPER_UNITY: f32 = 1000.0;
const TEMPERATURE: f32 = 333.15;
const Z_ELECTRON: f32 = 1.0;

struct ElectrochemicalModel {
    diffusion_number: f32,
    rate_constant_positive: f32,
    rate_constant_negative: f32,

    membrane_surface_area: f32,
    membrane_thickness: f32,
    stack_resistance: f32,
    time_step: f32,

    anolyte_c1: f32,
    anolyte_c2: f32,

    catholyte_c1: f32,
    catholyte_c0: f32,

    current_i: f32,

    voltage: f32,
    cycle: f32,

    charge_offset: f32,
    discharge_offset:f32
}

impl ElectrochemicalModel {
    fn time_step (&mut self) {

        self.charge_discharge_check(); 
        self.current_component();
        self.diffusion_step();
        self.voltage_calc();
    }

    fn charge_discharge_check(&mut self) {
        // Check whether timestep can be performed or current sign flip needed
        if self.current_i > 0.0 {
            if self.anolyte_c1 < 0.0 {
                self.current_i = self.current_i * -1.0;
                self.anolyte_c1 = 0.0;
                self.cycle += 1.0
            } else if self.catholyte_c1 < 0.0 {
                self.current_i = self.current_i * -1.0;
                self.catholyte_c1 = 0.0;
                self.cycle += 1.0
            }
        } else {
            if self.anolyte_c2 < 0.0 {
                self.current_i = self.current_i * -1.0;
                self.anolyte_c2 = 0.0;
                self.cycle += 1.0
            } else if self.catholyte_c0 < 0.0 {
               self.current_i = self.current_i * -1.0; 
               self.catholyte_c0 = 0.0;
               self.cycle += 1.0
            }
        }

        // Negative concentration check
        if self.anolyte_c1 < 0.0 {
            self.anolyte_c1 = 0.0;
        }
        if self.anolyte_c2 < 0.0 {
            self.anolyte_c2 = 0.0;
        }
        if self.catholyte_c0 < 0.0 {
            self.catholyte_c0 = 0.0;
        }
        if self.catholyte_c1 < 0.0 {
            self.catholyte_c1 = 0.0;
        }
    }

    fn current_component(&mut self) {
        let current_part = (1.0 / (Z_ELECTRON * FARADAY_CONSTANT) * self.current_i) * self.time_step / ELECTROLYTE_VOLUME;

        self.anolyte_c1 -= current_part;
        self.anolyte_c2 += current_part;

        self.catholyte_c1 -= current_part;
        self.catholyte_c0 += current_part;
    }

    fn diffusion_step(&mut self) {
        // C2 diffusion only occurs if there is C2 present in the anolyte, followed by comproportionation
        if self.anolyte_c2 > 0.0 {
            let diffusion_amount = self.diffusion_number * (self.membrane_surface_area / self.membrane_thickness) * self.time_step / ELECTROLYTE_VOLUME;

            self.catholyte_c1 += 2.0 * diffusion_amount * self.anolyte_c2;
            self.catholyte_c0 -= diffusion_amount * self.anolyte_c2;
            self.anolyte_c2 -=  diffusion_amount * self.anolyte_c2;
        }
    }

    fn voltage_calc(&mut self) {
        // Butler-volmer overpotentials
        // Exchange current densities from estimated rate constant
        let jp: f32 = 1.0 / self.membrane_surface_area * (FARADAY_CONSTANT * self.rate_constant_positive * self.anolyte_c2.powf(0.5) * self.anolyte_c1.powf(0.5));
        let jn: f32 = 1.0 / self.membrane_surface_area * (FARADAY_CONSTANT * self.rate_constant_negative * self.catholyte_c1.powf(0.5) * COPPER_UNITY.powf(0.5));
        
        // log term of Equation 9
        let logterm_positive = 1.0 /(2.0 * jp * self.membrane_surface_area) * self.current_i + ((1.0 / (2.0 * jp * self.membrane_surface_area) * self.current_i).powf(2.0) + 1.0 ).powf(0.5);
        // log term of Equation 10
        let logterm_negative = 1.0 /(2.0 * jn * self.membrane_surface_area) * self.current_i + ((1.0 / (2.0 * jn * self.membrane_surface_area) * self.current_i).powf(2.0) + 1.0 ).powf(0.5);

        // Positive overpotential of Equation 9
        let positive_overpotential = ((2.0 * GAS_CONSTANT * TEMPERATURE) / FARADAY_CONSTANT) * logterm_positive.ln();
        // Negative overpotential of Equation 10
        let negative_overpotential = ((2.0 * GAS_CONSTANT * TEMPERATURE) / FARADAY_CONSTANT) * logterm_negative.ln();

        let butler_volmer_overpotential = positive_overpotential - negative_overpotential;

        let nernst_overpotential = (GAS_CONSTANT * TEMPERATURE) / (Z_ELECTRON * FARADAY_CONSTANT) * ((self.anolyte_c2 * COPPER_UNITY) / (self.anolyte_c1 * self.catholyte_c1)).ln();

        // Stack resistance overpotentials
        let stack_overpotential = self.stack_resistance * self.current_i;

        let mut voltage_offset = 0.0;

        if self.current_i > 0.0 {
            voltage_offset = self.charge_offset;
        } else {
            voltage_offset = self.discharge_offset;
        }

        // System potenial
        self.voltage = butler_volmer_overpotential + nernst_overpotential + stack_overpotential + voltage_offset + FORMAL_POTENTIAL;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut electrochem_model = ElectrochemicalModel {
        diffusion_number: 0.0000000000019031356, 
        rate_constant_positive: 0.000007846213,
        rate_constant_negative: 0.000007680297,

        membrane_surface_area: 1e-4,
        membrane_thickness: 33e-6,
        stack_resistance: 0.04841762,
        time_step: 120.0,

        anolyte_c1: 1077.5035,
        anolyte_c2: 0.0,
        
        catholyte_c1: 1134.5563,
        catholyte_c0: 0.0,

        current_i: 20e-3,

        voltage: 0.0,
        cycle: 0.0,

        charge_offset: 0.13248,
        discharge_offset: -0.1519
    };

    // Arrays to capture data for plotting
    let mut voltage_data = Vec::new();

    let mut catholyte_c1_data = Vec::new();
    let mut catholyte_c0_data = Vec::new();

    let mut anolyte_c1_data = Vec::new();
    let mut anolyte_c2_data = Vec::new();

    let mut time_data = Vec::new();

    let mut real_current: Vec<f32> = Vec::new();


    // Import real data
    let mut rdr = Reader::from_path("data.csv")?;

    for result in rdr.records() {
        let record = result?;
        let record = &record[2];

        real_current.push(record.parse::<f32>().unwrap())           
    }

    // Tracking simulation time
    let mut time_counter:f32 = 0.0;

    for current in real_current {
        electrochem_model.current_i = current;
        electrochem_model.time_step();

        if time_counter % 10.0 == 0.0 {
            voltage_data.push(electrochem_model.voltage);
            catholyte_c1_data.push(electrochem_model.catholyte_c1);
            catholyte_c0_data.push(electrochem_model.catholyte_c0);

            anolyte_c1_data.push(electrochem_model.anolyte_c1);
            anolyte_c2_data.push(electrochem_model.anolyte_c2);

            time_data.push(time_counter);
        }

        time_counter += 120.0; 
    }

    println!("Time simulated, s: {}, m: {:.0}, h: {:.0}", time_counter, time_counter/60.0, time_counter/3600.0);

    let file = File::create("output.csv").expect("Unable to create file");
    let mut writer = BufWriter::new(&file);

    let mut counter = 0;

    while counter < voltage_data.len() {
        if counter == 0 {
            writeln!(writer, "Time, Voltage, c1c, c0c, c1a, c2a").expect("Failed to write data");
        }
        writeln!(writer, "{}, {}, {}, {}, {}, {}", 
        time_data[counter], voltage_data[counter], 
        catholyte_c1_data[counter], catholyte_c0_data[counter], 
        anolyte_c1_data[counter], anolyte_c2_data[counter])
        .expect("Failed to write data");
        
        counter += 1;
    }

    Ok(())
}


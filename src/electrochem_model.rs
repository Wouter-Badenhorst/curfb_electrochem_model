use std::io::BufWriter;
use std::io::Write;                                                                                                                                                                                                                                                                                                                           
use std::fs::File; 

const ELECTROLYTE_VOLUME: f32 = 0.06; 
const TEMPERATURE: f32 = 333.15;
const CELLS: f32 = 30.0;

const FARADAY_CONSTANT: f32 = 96485.0;
const FORMAL_POTENTIAL: f32 = 0.65;
const GAS_CONSTANT: f32 = 8.3145;
const COPPER_UNITY: f32 = 1000.0;
const Z_ELECTRON: f32 = 1.0;


#[allow(dead_code)]
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
                self.anolyte_c1 = 0.0;
                self.cycle += 1.0
            } else if self.catholyte_c1 < 0.0 {
                self.catholyte_c1 = 0.0;
                self.cycle += 1.0
            }
        } else {
            if self.anolyte_c2 < 0.0 {
                self.anolyte_c2 = 0.0;
                self.cycle += 1.0
            } else if self.catholyte_c0 < 0.0 {
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
        let current_part = (1.0 / (Z_ELECTRON * FARADAY_CONSTANT) * self.current_i * CELLS) * self.time_step / ELECTROLYTE_VOLUME;

        self.anolyte_c1 -= current_part;
        self.anolyte_c2 += current_part;

        self.catholyte_c1 -= current_part;
        self.catholyte_c0 += current_part;
    }

    fn diffusion_step(&mut self) {
        let diffusion_factor = self.diffusion_number * (self.membrane_surface_area * CELLS / self.membrane_thickness) * self.time_step / ELECTROLYTE_VOLUME;

        // C2 diffusion (from anolyte to catholyte)
        let c2_gradient = self.anolyte_c2 - 0.0; // Assuming no C2 in catholyte
        if c2_gradient > 0.0 {
            let c2_diffusion = diffusion_factor * c2_gradient;
            self.catholyte_c1 += 2.0 * c2_diffusion;
            self.catholyte_c0 -= c2_diffusion;
            self.anolyte_c2 -= c2_diffusion;
        }

        // C1 back diffusion (from catholyte to anolyte)
        let c1_gradient = self.catholyte_c1 - self.anolyte_c1;
        if c1_gradient != 0.0 {
            let c1_diffusion = diffusion_factor * c1_gradient;
            self.catholyte_c1 -= c1_diffusion;
            self.anolyte_c1 += c1_diffusion;
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

        let voltage_offset:f32;

        if self.current_i > 0.0 {
            voltage_offset = self.charge_offset;
        } else {
            voltage_offset = self.discharge_offset;
        }

        // System potenial
        self.voltage = (butler_volmer_overpotential + nernst_overpotential + FORMAL_POTENTIAL + voltage_offset) * CELLS + stack_overpotential ;
    }

}

pub fn electrochem_model_sim(file_write: bool, individual: [f64; 12], real_current: Vec<f32>, real_voltage: Vec<f32>) -> f64 {

    let mut electrochem_model = ElectrochemicalModel {
        diffusion_number: individual[5] as f32, 
        rate_constant_positive: individual[3] as f32,
        rate_constant_negative: individual[4] as f32,

        membrane_surface_area: 28.0 / 100.0 * 32.0 / 100.0,
        membrane_thickness: 60e-6,
        stack_resistance: individual[2] as f32,
        time_step: 60.0,

        anolyte_c1: individual[0] as f32,
        anolyte_c2: individual[8] as f32,
        
        catholyte_c1: individual[1] as f32,
        catholyte_c0: individual[9] as f32,

        current_i: 23.0,

        voltage: 0.0,
        cycle: 0.0,

        charge_offset: individual[6] as f32,
        discharge_offset: individual[7] as f32
    };

    // Arrays to capture data for plotting
    let mut voltage_data = Vec::new();

    let mut catholyte_c1_data = Vec::new();
    let mut catholyte_c0_data = Vec::new();

    let mut anolyte_c1_data = Vec::new();
    let mut anolyte_c2_data = Vec::new();

    let mut time_data = Vec::new();

    // Tracking simulation time
    let mut time_counter:f32 = 0.0;

    for current in real_current {
        electrochem_model.current_i = current;
        electrochem_model.time_step();

        voltage_data.push(electrochem_model.voltage);
        catholyte_c1_data.push(electrochem_model.catholyte_c1);
        catholyte_c0_data.push(electrochem_model.catholyte_c0);

        anolyte_c1_data.push(electrochem_model.anolyte_c1);
        anolyte_c2_data.push(electrochem_model.anolyte_c2);

        time_data.push(time_counter);

        time_counter += 60.0; 
    }

    let fitness = fitness_function(time_data.clone(), real_voltage.clone(), voltage_data.clone());

    if file_write {
        write_output(time_data, voltage_data, catholyte_c1_data, catholyte_c0_data, anolyte_c1_data, anolyte_c2_data);
    }

    fitness
    
}

fn fitness_function(time: Vec<f32>, real_voltage: Vec<f32>, simulated_voltage: Vec<f32>) -> f64 {

    let mut fitness: f64 = 0.0;

    for index in 0..time.len() -1 {
        let real_int = ((real_voltage[index] + real_voltage[index + 1]) / 2.0) * (time[index + 1] - time[index]);
        let simulated_int = ((simulated_voltage[index] + simulated_voltage[index + 1]) / 2.0) * (time[index + 1] - time[index]);

        fitness += ((real_int - simulated_int).powf(2.0)) as f64;
    }

    fitness
}

fn write_output(time_data: Vec<f32>, voltage_data: Vec<f32>, catholyte_c1_data: Vec<f32>, catholyte_c0_data: Vec<f32>, anolyte_c1_data: Vec<f32>, anolyte_c2_data: Vec<f32>) {
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
}

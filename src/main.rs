mod data_preparation;
mod electrochem_model;
mod genetic_algorithm;

use electrochem_model::electrochem_model_sim;
use crate::data_preparation::process_data;
use crate::genetic_algorithm::Population;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use csv::Reader;

fn read_real_data() -> (Vec<f32>, Vec<f32>) {
    // Import real data to use in the model
    let mut real_current: Vec<f32> = Vec::new();
    let mut real_voltage: Vec<f32> = Vec::new();

    let mut rdr = Reader::from_path("data.csv").unwrap();

    for result in rdr.records() {
        let record = result.unwrap();

        real_current.push(record[2].parse::<f32>().unwrap());
        real_voltage.push(record[1].parse::<f32>().unwrap());  
    }

    return (real_current, real_voltage)
}

fn main() {
    // Prepare data
    let input_file = "input.csv";
    let output_file = "data.csv";  // This will be used by the main program

    match process_data(input_file, output_file) {
        Ok(_) => println!("Successfully processed {} into {}", input_file, output_file),
        Err(e) => eprintln!("Error processing data: {}", e),
    }

    // Initialize population
    let mut population = Population {
        // Fitness tracking variables
        best_fitness: 0.0,          // Best fitness found so far
        worst_fitness: 0.0,         // Worst fitness in current population
        average_fitness: 0.0,       // Average fitness of current population

        // Genetic algorithm parameters
        mutation_intensity: 0.3,   // Reduced for finer local search
        crossover_rate: 0.7,        // Increased for better gene mixing
        mutation_rate: 0.3,         // Balanced for exploration/exploitation
        elite_size: 0.1,           // Increased elite preservation

        individual_list: Vec::new(),

        // Model parameter bounds
        parameter_bounds_upper: [
            3000.0,         // [0] Anolyte concentration C1 (mol/m³)
            3000.0,         // [1] Catholyte concentration C1 (mol/m³)
            0.5,            // [2] Stack resistance (Ohm)
            1.0e0,          // [3] Positive rate constant k+ (m/s)
            1.0e0,          // [4] Negative rate constant k- (m/s)
            1.0e-10,        // [5] Membrane diffusion coefficient (m²/s)
            0.5,            // [6] Charge offset (V)
            0.5,            // [7] Discharge offset (V)
            1500.0,         // [8] Anolyte concentration C2 (mol/m³)
            1500.0,         // [9] Catholyte concentration C0 (mol/m³)
        ],
        parameter_bounds_lower: [
            1000.0,         // [0] Anolyte concentration C1 (mol/m³)
            1000.0,         // [1] Catholyte concentration C1 (mol/m³)
            0.0,            // [2] Stack resistance (Ohm)
            1.0e-8,         // [3] Positive rate constant k+ (m/s)
            1.0e-8,         // [4] Negative rate constant k- (m/s)
            1.0e-14,        // [5] Membrane diffusion coefficient (m²/s)
            -0.5,           // [6] Charge offset (V)
            -0.5,           // [7] Discharge offset (V)
            0.0,            // [8] Anolyte concentration C2 (mol/m³)
            0.0,            // [9] Catholyte concentration C0 (mol/m³)
        ],

        // Algorithm control
        maximum_generation: 50,         // More generations for better convergence
        current_generation: 0,          // Current generation counter
    };

    // Generate initial population
    population.generate_pop(500000);    // Population size

    let max_gen = population.maximum_generation;
    let mut cur_gen = population.current_generation;

    let shared_struct = Arc::new(Mutex::new(population));

    let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Grab the real current and voltage data, only single file read
    let (real_current, real_voltage) = read_real_data();

    while cur_gen < max_gen {
        // Use larger chunks for better parallel performance
        shared_struct.lock().unwrap().individual_list.par_chunks_mut(1000).for_each(|chunk| {
            chunk.iter_mut().for_each(|individual| {
                individual[10] = electrochem_model_sim(false, *individual, real_current.clone(), real_voltage.clone());
            });
        });

        let mut population = shared_struct.lock().unwrap();

        // Preserve best solutions before modification
        population.preserve_best_solutions();
        population.population_crossover();
        population.mutate_population();
        
        // Preserve best solutions after modification
        population.preserve_best_solutions();
        
        population.current_generation += 1;
        cur_gen += 1;

        let best_individual = population.best_fitness_calc();
        let best_params = population.individual_list[best_individual];
        
        // Only write output in the final generation
        if cur_gen == max_gen {
            // Run simulation one final time with output writing enabled for plotting
            electrochem_model_sim(
                true,  // Enable file writing
                best_params,
                real_current.clone(),
                real_voltage.clone()
            );
        }

        drop(population);
    }

    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    println!("Total duration: {} s", (end_time - start_time));
}

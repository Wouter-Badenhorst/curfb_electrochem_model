mod electrochem_model;


use electrochem_model::electrochem_model_sim;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::Value;
use rand::Rng;
use std::fs;

use std::sync::{Arc, Mutex};
use std::thread;

#[allow(dead_code)]
struct Population {
    best_fitness: f64,
    worst_fitness: f64,
    average_fitness: f64,

    mutation_intensity: f64,
    crossover_rate: f64,
    mutation_rate: f64,
    elite_size: f64,

    individual_list: Vec<[f64; 10]>,

    parameter_bounds_upper: [f64; 8],
    parameter_bounds_lower: [f64; 8],

    current_generation: u64,
    maximum_generation: u64

}

#[allow(dead_code)]
impl Population {
    fn generate_pop(&mut self, pop_size: u64) {
        // Assign unique identifier to each individual for later multithreadings
        let mut identifier = 0.0;

        while self.individual_list.len() <= pop_size.try_into().unwrap() {
            let mut individual = self.random_population();
            individual[9] = identifier;
            self.individual_list.push(individual);

            identifier += 1.0;
        }
    }

    fn random_population (&mut self) -> [f64; 10] {
        // Create random first generation values
        let mut rng = rand::thread_rng();
        let mut individual = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let mut index = 0;

        while index < individual.len() - 2 {
            let new_gene = rng.gen_range(self.parameter_bounds_lower[index]..self.parameter_bounds_upper[index]);
            individual[index] = new_gene;
            index += 1;
        }

        individual
    }

    fn population_crossover (&mut self) {

        // TODO: Makes less janky, less than intended amount of elites are created
        // This is by elites being overwritten by better elites before the empty array is filled
        // Additionally crossover rate is not properly implemented

        // Find indexes of elite population limited by elite_size
        let max_elites: f64 = self.individual_list.len() as f64 * self.elite_size;
        let mut elite_count: f64 = 0.0;

        let mut elites: Vec<[f64; 2]> = Vec::new();

        while elite_count < max_elites {
            elites.push([f64::INFINITY, f64::INFINITY]);
            elite_count += 1.0;
        }    

        for index in 0..self.individual_list.len() {

            for elite in elites.iter_mut() {

                if self.individual_list[index][8] < elite[1]{
                    
                    elite [1] = self.individual_list[index][8];
                    elite [0] = index as f64;

                    break;
                }
            }

            elites[1].sort_by(|a, b| a.partial_cmp(b).unwrap());
        }

        // Perform population crossover using the elites
        let mut rng = rand::thread_rng();
        let mut index_counter = 0;

        // TODO: Fix this
        // Duplicate elites to fill in void
        let mut number_of_elites = 0;

        for index in 0..elites.len() {

            if elites[index][0] != f64::INFINITY {
                number_of_elites += 1;
            } else {
                let random_index = rng.gen_range(0..number_of_elites);
                elites[index][0] = elites[random_index][0];
                elites[index][1] = elites[random_index][1];
            }
        }

        for elite in elites {
            let random_index =rng.gen_range(0..self.individual_list.len());

            for gene in self.individual_list[elite[0] as usize].clone().iter() {

                let alpha_fitness = elite[1];

                let beta_param = self.individual_list[random_index][index_counter].clone();
                let beta_fitness = self.individual_list[random_index][8].clone();

                let gene_mutated = ((gene * alpha_fitness)+(beta_param * beta_fitness)) / ( alpha_fitness + beta_fitness);

                self.individual_list[random_index][index_counter] = gene_mutated;

                index_counter += 1;

                if index_counter == 7 {
                    break;
                }
            }

            index_counter = 0;
        }



    }

    fn mutate_population (&mut self) {
        // Gene index counter
        let mut rng = rand::thread_rng();

        for individual in self.individual_list.iter_mut() {

            for index in 0..7 {
                // Checks whether to mutate
                let mut noise = 0.0;

                if rng.gen_range(0.0..1.0) < self.mutation_rate {
                    noise = ((1.0 - (self.current_generation / self.maximum_generation) as f64)  * self.mutation_intensity) * (self.parameter_bounds_upper[index] - self.parameter_bounds_lower[index]);
                } 

                if noise > 0.0 {

                    let lower_gene = individual[index] - noise;
                    let upper_gene = individual[index] + noise;

                    if lower_gene >= upper_gene {
                        break;
                    } else if lower_gene.is_nan() || upper_gene.is_nan() {
                        break;
                    }

                    let mut new_gene = randomize_gene(lower_gene, upper_gene);

                    if new_gene < self.parameter_bounds_lower[index] {
                        new_gene = self.parameter_bounds_lower[index];
                    } else if new_gene > self.parameter_bounds_upper[index] {
                        new_gene = self.parameter_bounds_upper[index];
                    }
                    
                    individual[index] = new_gene;
                } 
            }
        }
            
    }

    fn best_fitness_calc (&mut self) {
        let mut best_fitness = f64::INFINITY;
        let mut best_individual = 0;

        let mut index = 0;

        for individual in self.individual_list.iter() {

            if individual[8] < best_fitness {
                best_fitness = individual[8];
                best_individual = index;
            }

            index += 1;
        }

        self.best_fitness = best_fitness;

        println!("###################################################################################################");
        println!("Generation: {:.0}, Fitness: {:.2}, C1a: {:.2}, C1c: {:.2}, R: {:.2}, k+ (e-1): {:.2}, k- (1e-5): {:.2}, Dmem (1e-12): {:.2}, Vc: {:.3}, Vd: {:.3}",
        self.current_generation,
        self.best_fitness / 10000.0,
        self.individual_list[best_individual][0],
        self.individual_list[best_individual][1],
        self.individual_list[best_individual][2],
        self.individual_list[best_individual][3] / 1e-1,
        self.individual_list[best_individual][4] / 1e-5,
        self.individual_list[best_individual][5] / 1e-12,
        self.individual_list[best_individual][6],
        self.individual_list[best_individual][7]);

    }

}

fn randomize_gene(lower: f64, upper: f64) -> f64 {
    let mut rng = rand::thread_rng();

    let new_gene = rng.gen_range(lower..upper);

    new_gene
}

fn main() {

    // Load setttings
    let data = fs::read_to_string("population_specification.json").unwrap();
    let settings: Value = serde_json::from_str(&data).unwrap();

    // Load the population struct
    let mut population = Population {
        best_fitness: settings["best_fitness"].as_f64().unwrap(),
        worst_fitness: settings["worst_fitness"].as_f64().unwrap(),
        average_fitness: settings["average_fitness"].as_f64().unwrap(),

        mutation_intensity: settings["mutation_intensity"].as_f64().unwrap(),
        crossover_rate: settings["crossover_rate"].as_f64().unwrap(),
        mutation_rate: settings["mutation_rate"].as_f64().unwrap(),
        elite_size: settings["elite_size"].as_f64().unwrap(),

        individual_list: Vec::new(),
        parameter_bounds_upper: [0.0; 8],
        parameter_bounds_lower: [0.0; 8],

        maximum_generation: settings["max_generation"].as_u64().unwrap(),
        current_generation: 0,

    };

    // TODO: Find a better solution
    // Assign upper and lower bounds
    let mut index = 0;
    if let serde_json::Value::Array(bounds) = &settings["lower_bounds"] {
        for bound in bounds {
            population.parameter_bounds_lower[index] = bound.as_f64().unwrap();
            index += 1;
        }
        index = 0;
    }
    if let serde_json::Value::Array(bounds) = &settings["upper_bounds"] {
        for bound in bounds {
            population.parameter_bounds_upper[index] = bound.as_f64().unwrap();
            index += 1;
        }
    }

    // Generate population
    population.generate_pop(settings["population_size"].as_u64().unwrap());

    let max_gen = population.maximum_generation;
    let mut cur_gen = population.current_generation;

    let shared_struct = Arc::new(Mutex::new(population));

    let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    while cur_gen < max_gen {

        // Calculate fitness using multithreading


        let mut threads = vec![];

        for i in 0..shared_struct.lock().unwrap().individual_list.len() {
            let shared_struct = shared_struct.clone();
            
            let thread_handle = thread::spawn(move || {
                let mut my_struct = shared_struct.lock().unwrap();

                let fitness = electrochem_model_sim(false, my_struct.individual_list[i]);
                my_struct.individual_list[i][8] = fitness;
            });

            threads.push(thread_handle);
        }

        for thread_handle in threads {
            thread_handle.join().unwrap();
        }

        // Unlock the population from the Mutex
        let mut population = shared_struct.lock().unwrap();

        population.population_crossover();

        population.mutate_population();

        population.current_generation += 1;
        cur_gen += 1;

        population.best_fitness_calc();

    }

    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    println!("Total duration: {} s", (end_time - start_time));
    
}

mod electrochem_model;

use electrochem_model::electrochem_model_sim;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use serde_json::Value;
use csv::Reader;
use rand::Rng;
use std::fs;



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
            let new_gene = rng.gen_range(self.parameter_bounds_lower[index] as f64..self.parameter_bounds_upper[index] as f64);
            individual[index] = new_gene;
            index += 1;
        }

        individual
    }

    fn population_crossover (&mut self) {

        // Find indexes of elite population limited by elite_size
        let max_elites: f64 = self.individual_list.len() as f64 * self.elite_size;
        let mut elite_count: f64 = 0.0;

        let mut elites: Vec<[f64; 2]> = Vec::new();

        while elite_count < max_elites {
            elites.push([f64::INFINITY, 0.0]);
            elite_count += 1.0;
        }    

        while elites[0].contains(&f64::INFINITY) {
            
            for index in 0..self.individual_list.len() - 1 {

                // Sort elites to make sure only worst candidates get removed
                elites.sort_by(|a, b| b.partial_cmp(a).unwrap());
                // elites.reverse();
    
                for elite in elites.iter_mut() {
    
                    if self.individual_list[index][8] < elite[0]{
                        
                        elite [0] = self.individual_list[index][8];
                        elite [1] = self.individual_list[index][9];
    
                        break;
                    }
                }
            }
        }        

        // Perform population crossover using the elites
        let mut rng = rand::thread_rng();
        let mut index_counter = 0;

        for elite in elites {
            let random_index =rng.gen_range(0..self.individual_list.len() - 1);


            for gene in self.individual_list[elite[1] as usize].clone().iter() {

                let alpha_fitness = elite[0];

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
       

        self.individual_list.par_iter_mut().for_each(|individual| 

            for index in 0..7 {
                // Checks whether to mutate
                let mut noise = 0.0;

                if random_helper() < self.mutation_rate {
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
        )
            
    }

    fn best_fitness_calc (&mut self) -> usize {
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

        best_individual
    }
}

fn randomize_gene(lower: f64, upper: f64) -> f64 {
    let mut rng = rand::thread_rng();

    let new_gene = rng.gen_range(lower..upper);

    new_gene
}

fn random_helper() -> f64 {
    let mut rng = rand::thread_rng();

    rng.gen_range(0.0..1.0)
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

    // Grab the real current and voltage data, only single file read
    let (real_current, real_voltage) = read_real_data();

    while cur_gen < max_gen {

        shared_struct.lock().unwrap().individual_list.par_iter_mut().for_each(|individual| 
            individual[8] = electrochem_model_sim(false, *individual, real_current.clone(), real_voltage.clone())

        );

        let mut population = shared_struct.lock().unwrap();

        population.population_crossover();

        population.mutate_population();

        population.current_generation += 1;
        cur_gen += 1;

        let best_individual = population.best_fitness_calc();

        let _fitness = electrochem_model_sim(true, population.individual_list[best_individual], real_current.clone(), real_voltage.clone());

    }

    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    println!("Total duration: {} s", (end_time - start_time));
    
}


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
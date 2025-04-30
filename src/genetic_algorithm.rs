use rand::Rng;
use rayon::prelude::*;

#[derive(Clone)]
pub struct Population {
    pub best_fitness: f64,
    pub worst_fitness: f64,
    pub average_fitness: f64,

    pub mutation_intensity: f64,
    pub crossover_rate: f64,
    pub mutation_rate: f64,
    pub elite_size: f64,

    pub individual_list: Vec<[f64; 12]>,
    pub parameter_bounds_upper: [f64; 10],
    pub parameter_bounds_lower: [f64; 10],

    pub current_generation: u64,
    pub maximum_generation: u64
}

impl Population {
    pub fn generate_pop(&mut self, pop_size: u64) {
        // Assign unique identifier to each individual for later multithreadings
        let mut identifier = 0.0;

        while self.individual_list.len() <= pop_size.try_into().unwrap() {
            let mut individual = self.random_population();
            individual[11] = identifier;
            self.individual_list.push(individual);

            identifier += 1.0;
        }
    }

    fn random_population(&mut self) -> [f64; 12] {  // Changed return type from 11 to 12
        let mut rng = rand::thread_rng();
        let mut individual = [0.0; 12];  // Initialize with correct size

        let mut index = 0;
        while index < self.parameter_bounds_upper.len() {
            let new_gene = rng.gen_range(self.parameter_bounds_lower[index]..self.parameter_bounds_upper[index]);
            individual[index] = new_gene;
            index += 1;
        }
        
        // Initialize fitness and identifier
        individual[10] = f64::INFINITY;  // Fitness value
        individual[11] = 0.0;            // Identifier

        individual
    }

    pub fn population_crossover(&mut self) {
        let elite_indices = self.get_elite_indices();
        let (_, adaptive_crossover) = self.get_adaptive_rates();
        
        // Sort population by fitness
        let mut sorted_indices: Vec<(usize, f64)> = self.individual_list.iter()
            .enumerate()
            .map(|(i, ind)| (i, ind[10]))
            .collect();
        
        sorted_indices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Rank-based selection probabilities
        let weights: Vec<f64> = (0..self.individual_list.len())
            .map(|i| 1.0 / (1.0 + i as f64))
            .collect();
        
        for &elite_idx in &elite_indices {
            if random_helper() < adaptive_crossover {
                // Select parent based on rank probability
                let parent_idx = Self::select_by_rank(&weights);
                let parent = self.individual_list[sorted_indices[parent_idx].0];
                
                // Perform crossover
                for index in 0..10 {
                    if random_helper() < 0.5 {
                        self.individual_list[elite_idx][index] = parent[index];
                    }
                }
            }
        }
    }

    fn select_by_rank(weights: &[f64]) -> usize {
        let total: f64 = weights.iter().sum();
        let mut r = random_helper() * total;
        
        for (i, &weight) in weights.iter().enumerate() {
            r -= weight;
            if r <= 0.0 {
                return i;
            }
        }
        
        weights.len() - 1
    }

    fn get_adaptive_rates(&self) -> (f64, f64) {
        let progress = self.current_generation as f64 / self.maximum_generation as f64;
        
        // Mutation rate decreases from initial value to 10% of initial value
        let adaptive_mutation = self.mutation_rate * (1.0 - 0.9 * progress);
        
        // Crossover rate increases as mutation decreases
        let adaptive_crossover = self.crossover_rate * (0.5 + 0.5 * progress);
        
        (adaptive_mutation, adaptive_crossover)
    }

    pub fn mutate_population(&mut self) {
        let elite_count = (self.individual_list.len() as f64 * self.elite_size) as usize;
        let (adaptive_mutation, _) = self.get_adaptive_rates();
        let best_fitness = self.best_fitness;

        // Use larger chunks for better parallel performance
        self.individual_list[elite_count..].par_chunks_mut(1000).for_each(|chunk| {
            let mut rng = rand::thread_rng(); // Create RNG per thread
            
            for individual in chunk {
                if best_fitness.is_finite() {
                    let fitness_factor = if individual[10].is_finite() {
                        (individual[10] / best_fitness).min(2.0)
                    } else {
                        2.0
                    };
                    
                    let intensity = self.mutation_intensity * fitness_factor;
                    
                    // Batch random number generation
                    if rng.gen::<f64>() < adaptive_mutation {
                        for index in 0..10 {
                            let range = self.parameter_bounds_upper[index] - self.parameter_bounds_lower[index];
                            let noise = intensity * range * (rng.gen::<f64>() * 2.0 - 1.0);
                            individual[index] = (individual[index] + noise)
                                .clamp(self.parameter_bounds_lower[index], 
                                      self.parameter_bounds_upper[index]);
                        }
                    }
                }
            }
        });
    }

    pub fn best_fitness_calc(&mut self) -> usize {
        let mut best_fitness = f64::INFINITY;
        let mut best_individual = 0;
        let mut index = 0;

        for individual in self.individual_list.iter() {
            if individual[10] < best_fitness {  
                best_fitness = individual[10];   
                best_individual = index;
            }
            index += 1;
        }

        self.best_fitness = best_fitness;

        self.update_population_stats();

        println!("+----------------+-------------+-------------+");
        println!("| Generation     | {:>11} |             |", self.current_generation);
        println!("+----------------+-------------+-------------+");
        println!("| Best Fitness   | {:>11.2} |             |", self.best_fitness / 10000.0);
        println!("| Worst Fitness  | {:>11.2} |             |", self.worst_fitness / 10000.0);
        println!("| Avg Fitness    | {:>11.2} |             |", self.average_fitness / 10000.0);
        println!("+----------------+-------------+-------------+");
        println!("| Parameter      | Value       | % of Upper  |");
        println!("+----------------+-------------+-------------+");
        println!("| C1a (mol/m³)   | {:>11.2} | {:>11.2} |", self.individual_list[best_individual][0], (self.individual_list[best_individual][0] / self.parameter_bounds_upper[0]) * 100.0);
        println!("| C2a (mol/m³)   | {:>11.2} | {:>11.2} |", self.individual_list[best_individual][8], (self.individual_list[best_individual][8] / self.parameter_bounds_upper[8]) * 100.0);
        println!("| C0c (mol/m³)   | {:>11.2} | {:>11.2} |", self.individual_list[best_individual][9], (self.individual_list[best_individual][9] / self.parameter_bounds_upper[9]) * 100.0);
        println!("| C1c (mol/m³)   | {:>11.2} | {:>11.2} |", self.individual_list[best_individual][1], (self.individual_list[best_individual][1] / self.parameter_bounds_upper[1]) * 100.0);
        println!("| R (Ohm)        | {:>11.2} | {:>11.2} |", self.individual_list[best_individual][2], (self.individual_list[best_individual][2] / self.parameter_bounds_upper[2]) * 100.0);
        println!("| k+ (1e-6 m/s)  | {:>11.2} | {:>11.2} |", self.individual_list[best_individual][3] / 1e-6, (self.individual_list[best_individual][3] / self.parameter_bounds_upper[3]) * 100.0);
        println!("| k- (1e-6 m/s)  | {:>11.2} | {:>11.2} |", self.individual_list[best_individual][4] / 1e-6, (self.individual_list[best_individual][4] / self.parameter_bounds_upper[4]) * 100.0);
        println!("| Dmem (1e-12)   | {:>11.2} | {:>11.2} |", self.individual_list[best_individual][5] / 1e-12, (self.individual_list[best_individual][5] / self.parameter_bounds_upper[5]) * 100.0);
        println!("| Vc (V)         | {:>11.3} | {:>11.2} |", self.individual_list[best_individual][6], (self.individual_list[best_individual][6] / self.parameter_bounds_upper[6]) * 100.0);
        println!("| Vd (V)         | {:>11.3} | {:>11.2} |", self.individual_list[best_individual][7], (self.individual_list[best_individual][7] / self.parameter_bounds_upper[7]) * 100.0);
        println!("+----------------+-------------+-------------+");

        best_individual
    }

    fn get_elite_indices(&self) -> Vec<usize> {
        let elite_count = (self.individual_list.len() as f64 * self.elite_size) as usize;
        
        // Create sorted indices using fitness value at index 10
        let mut sorted_indices: Vec<(usize, f64)> = self.individual_list.iter()
            .enumerate()
            .map(|(i, ind)| (i, ind[10])) // Changed from ind[8] to ind[10] for fitness value
            .collect();
        
        // Sort by fitness (lower is better) with NaN handling
        sorted_indices.sort_by(|a, b| {
            match (a.1.is_finite(), b.1.is_finite()) {
                (true, true) => a.1.partial_cmp(&b.1)
                    .unwrap_or(std::cmp::Ordering::Equal),
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (false, false) => std::cmp::Ordering::Equal,
            }
        });
        
        // Return indices of elite individuals (best performers)
        sorted_indices.iter()
            .take(elite_count)
            .map(|(i, _)| *i)
            .collect()
    }

    pub fn preserve_best_solutions(&mut self) {
        // Pre-allocate sorted indices to avoid reallocation
        let len = self.individual_list.len();
        let mut indices: Vec<usize> = (0..len).collect();
        
        // Sort indices instead of moving whole arrays
        indices.sort_unstable_by(|&a, &b| {
            match (self.individual_list[a][10].is_finite(), self.individual_list[b][10].is_finite()) {
                (true, true) => self.individual_list[a][10]
                    .partial_cmp(&self.individual_list[b][10])
                    .unwrap_or(std::cmp::Ordering::Equal),
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (false, false) => std::cmp::Ordering::Equal,
            }
        });

        // Update best fitness if valid
        if self.individual_list[indices[0]][10].is_finite() {
            self.best_fitness = self.individual_list[indices[0]][10];
        }

        // Preserve elites more efficiently
        let elite_count = (len as f64 * self.elite_size) as usize;
        let mut new_population = Vec::with_capacity(len);
        
        // Add elites first
        for &idx in indices.iter().take(elite_count) {
            new_population.push(self.individual_list[idx]);
        }
        
        // Add remaining individuals
        for &idx in indices.iter().skip(elite_count) {
            new_population.push(self.individual_list[idx]);
        }

        self.individual_list = new_population;
        self.update_population_stats();
    }

    // Add new method to calculate population statistics
    fn update_population_stats(&mut self) {
        // Use parallel iterator for large populations
        let (sum, worst) = self.individual_list.par_iter()
            .map(|individual| {
                let fitness = individual[10];
                if fitness.is_finite() {
                    (fitness, fitness)
                } else {
                    (0.0, f64::NEG_INFINITY)
                }
            })
            .reduce(
                || (0.0, f64::NEG_INFINITY),
                |a, b| (a.0 + b.0, a.1.max(b.1))
            );
        
        self.worst_fitness = worst;
        self.average_fitness = sum / self.individual_list.len() as f64;
    }
}

pub fn random_helper() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..1.0)
}
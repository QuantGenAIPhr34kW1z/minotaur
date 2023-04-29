//! NSGA-II Multi-Objective Optimization Algorithm
//!
//! Implementation of the Non-dominated Sorting Genetic Algorithm II (NSGA-II)
//! for multi-objective turbofan cycle optimization.
//!
//! References:
//! - Deb, K., et al. "A Fast and Elitist Multiobjective Genetic Algorithm: NSGA-II" (2002)
//!
//! Author: CSTNSystems, EINIX SA
//! License: LicenseRef-EINIXSA-Internal-Eval
//! Version: 2.9.0

use std::cmp::Ordering;

/// Individual in the population
#[derive(Clone, Debug)]
pub struct Individual {
    /// Design variables: [bpr, opr, eta_comp, eta_turb]
    pub x: Vec<f64>,
    /// Objective values: [tsfc, -thrust] (minimize both)
    pub f: Vec<f64>,
    /// Constraint violations (sum of violations)
    pub cv: f64,
    /// Pareto rank (0 = non-dominated front)
    pub rank: usize,
    /// Crowding distance
    pub crowding_distance: f64,
    /// Solver status
    pub status: i32,
    /// Additional outputs: [t4, iterations]
    pub outputs: Vec<f64>,
}

impl Individual {
    pub fn new(x: Vec<f64>) -> Self {
        Self {
            x,
            f: Vec::new(),
            cv: 0.0,
            rank: usize::MAX,
            crowding_distance: 0.0,
            status: -1,
            outputs: Vec::new(),
        }
    }

    /// Check if this individual dominates another
    pub fn dominates(&self, other: &Individual) -> bool {
        // Handle constraint violations first
        if self.cv < other.cv {
            return true;
        }
        if self.cv > other.cv {
            return false;
        }

        // Both feasible or same constraint violation
        let mut dominated = false;
        for (a, b) in self.f.iter().zip(other.f.iter()) {
            if a > b {
                return false; // Worse in at least one objective
            }
            if a < b {
                dominated = true; // Better in at least one
            }
        }
        dominated
    }
}

/// NSGA-II configuration
#[derive(Clone, Debug)]
pub struct NSGA2Config {
    /// Population size (must be even)
    pub pop_size: usize,
    /// Number of generations
    pub generations: usize,
    /// Crossover probability
    pub crossover_prob: f64,
    /// Mutation probability (per gene)
    pub mutation_prob: f64,
    /// Distribution index for SBX crossover
    pub eta_c: f64,
    /// Distribution index for polynomial mutation
    pub eta_m: f64,
    /// Variable bounds: [(min, max), ...]
    pub bounds: Vec<(f64, f64)>,
    /// Seed for reproducibility
    pub seed: u64,
}

impl Default for NSGA2Config {
    fn default() -> Self {
        Self {
            pop_size: 100,
            generations: 50,
            crossover_prob: 0.9,
            mutation_prob: 0.1,
            eta_c: 20.0,
            eta_m: 20.0,
            bounds: vec![
                (0.2, 1.5),   // bpr
                (4.0, 16.0),  // opr
                (0.75, 0.90), // eta_comp
                (0.80, 0.92), // eta_turb
            ],
            seed: 42,
        }
    }
}

/// Pareto front result
#[derive(Clone, Debug)]
pub struct ParetoFront {
    /// Non-dominated solutions
    pub solutions: Vec<Individual>,
    /// Generation at which found
    pub generation: usize,
    /// Hypervolume indicator (if computed)
    pub hypervolume: Option<f64>,
}

/// NSGA-II optimizer
pub struct NSGA2 {
    config: NSGA2Config,
    population: Vec<Individual>,
    rng_state: u64,
}

impl NSGA2 {
    pub fn new(config: NSGA2Config) -> Self {
        Self {
            rng_state: config.seed,
            config,
            population: Vec::new(),
        }
    }

    /// Simple LCG random number generator (deterministic)
    fn rand(&mut self) -> f64 {
        // Linear congruential generator
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.rng_state >> 33) as f64) / ((1u64 << 31) as f64)
    }

    /// Generate random integer in [0, n)
    fn rand_int(&mut self, n: usize) -> usize {
        (self.rand() * n as f64) as usize
    }

    /// Initialize population with Latin Hypercube Sampling
    pub fn initialize_population(&mut self) {
        let n = self.config.pop_size;
        let d = self.config.bounds.len();

        // Simple LHS: divide each dimension into n intervals
        let mut indices: Vec<Vec<usize>> = (0..d)
            .map(|_| (0..n).collect())
            .collect();

        // Shuffle each dimension
        for dim in indices.iter_mut() {
            for i in (1..n).rev() {
                let j = self.rand_int(i + 1);
                dim.swap(i, j);
            }
        }

        // Create individuals
        self.population = Vec::with_capacity(n);
        for i in 0..n {
            let x: Vec<f64> = (0..d)
                .map(|j| {
                    let (lo, hi) = self.config.bounds[j];
                    let idx = indices[j][i] as f64;
                    let u = (idx + self.rand()) / n as f64;
                    lo + u * (hi - lo)
                })
                .collect();
            self.population.push(Individual::new(x));
        }
    }

    /// Evaluate population using provided objective function
    pub fn evaluate<F>(&mut self, eval_fn: &F)
    where
        F: Fn(&[f64]) -> (Vec<f64>, f64, i32, Vec<f64>),
    {
        for ind in self.population.iter_mut() {
            let (f, cv, status, outputs) = eval_fn(&ind.x);
            ind.f = f;
            ind.cv = cv;
            ind.status = status;
            ind.outputs = outputs;
        }
    }

    /// Fast non-dominated sorting
    pub fn non_dominated_sort(&mut self) {
        let n = self.population.len();

        // Reset ranks
        for ind in self.population.iter_mut() {
            ind.rank = usize::MAX;
        }

        // Domination counts and dominated sets
        let mut domination_count: Vec<usize> = vec![0; n];
        let mut dominated_by: Vec<Vec<usize>> = vec![Vec::new(); n];

        // Compare all pairs
        for i in 0..n {
            for j in (i + 1)..n {
                if self.population[i].dominates(&self.population[j]) {
                    dominated_by[i].push(j);
                    domination_count[j] += 1;
                } else if self.population[j].dominates(&self.population[i]) {
                    dominated_by[j].push(i);
                    domination_count[i] += 1;
                }
            }
        }

        // Find fronts
        let mut current_front: Vec<usize> = Vec::new();
        for (i, &count) in domination_count.iter().enumerate() {
            if count == 0 {
                self.population[i].rank = 0;
                current_front.push(i);
            }
        }

        let mut front_idx = 0;
        while !current_front.is_empty() {
            let mut next_front: Vec<usize> = Vec::new();
            for &i in &current_front {
                for &j in &dominated_by[i] {
                    domination_count[j] -= 1;
                    if domination_count[j] == 0 {
                        self.population[j].rank = front_idx + 1;
                        next_front.push(j);
                    }
                }
            }
            front_idx += 1;
            current_front = next_front;
        }
    }

    /// Calculate crowding distance for each front
    pub fn crowding_distance(&mut self) {
        let n = self.population.len();
        if n == 0 {
            return;
        }

        // Reset crowding distances
        for ind in self.population.iter_mut() {
            ind.crowding_distance = 0.0;
        }

        let n_obj = self.population[0].f.len();

        // Process each front separately
        let max_rank = self.population.iter().map(|i| i.rank).max().unwrap_or(0);
        for rank in 0..=max_rank {
            let mut front_indices: Vec<usize> = self.population
                .iter()
                .enumerate()
                .filter(|(_, ind)| ind.rank == rank)
                .map(|(i, _)| i)
                .collect();

            if front_indices.len() <= 2 {
                // Set infinite distance for boundary points
                for &i in &front_indices {
                    self.population[i].crowding_distance = f64::INFINITY;
                }
                continue;
            }

            // For each objective
            for m in 0..n_obj {
                // Sort by objective m
                front_indices.sort_by(|&a, &b| {
                    self.population[a].f[m]
                        .partial_cmp(&self.population[b].f[m])
                        .unwrap_or(Ordering::Equal)
                });

                // Boundary points get infinite distance
                let first = front_indices[0];
                let last = front_indices[front_indices.len() - 1];
                self.population[first].crowding_distance = f64::INFINITY;
                self.population[last].crowding_distance = f64::INFINITY;

                // Calculate range for normalization
                let f_min = self.population[first].f[m];
                let f_max = self.population[last].f[m];
                let range = if (f_max - f_min).abs() > 1e-10 {
                    f_max - f_min
                } else {
                    1.0
                };

                // Interior points
                for i in 1..(front_indices.len() - 1) {
                    let prev = front_indices[i - 1];
                    let next = front_indices[i + 1];
                    let curr = front_indices[i];
                    self.population[curr].crowding_distance +=
                        (self.population[next].f[m] - self.population[prev].f[m]) / range;
                }
            }
        }
    }

    /// Tournament selection
    fn tournament_select(&mut self) -> usize {
        let a = self.rand_int(self.population.len());
        let b = self.rand_int(self.population.len());

        // Compare by rank first, then crowding distance
        let ind_a = &self.population[a];
        let ind_b = &self.population[b];

        if ind_a.rank < ind_b.rank {
            a
        } else if ind_b.rank < ind_a.rank {
            b
        } else if ind_a.crowding_distance > ind_b.crowding_distance {
            a
        } else {
            b
        }
    }

    /// Simulated Binary Crossover (SBX)
    fn sbx_crossover(&mut self, p1: &[f64], p2: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let d = p1.len();
        let mut c1 = p1.to_vec();
        let mut c2 = p2.to_vec();

        if self.rand() > self.config.crossover_prob {
            return (c1, c2);
        }

        for i in 0..d {
            if self.rand() > 0.5 {
                continue;
            }

            let (lo, hi) = self.config.bounds[i];
            let y1 = p1[i].min(p2[i]);
            let y2 = p1[i].max(p2[i]);

            if (y2 - y1).abs() < 1e-10 {
                continue;
            }

            let beta = 1.0 + (2.0 * (y1 - lo) / (y2 - y1));
            let alpha = 2.0 - beta.powf(-(self.config.eta_c + 1.0));
            let u = self.rand();
            let betaq = if u <= 1.0 / alpha {
                (u * alpha).powf(1.0 / (self.config.eta_c + 1.0))
            } else {
                (1.0 / (2.0 - u * alpha)).powf(1.0 / (self.config.eta_c + 1.0))
            };

            c1[i] = 0.5 * ((y1 + y2) - betaq * (y2 - y1));
            c2[i] = 0.5 * ((y1 + y2) + betaq * (y2 - y1));

            // Bound enforcement
            c1[i] = c1[i].max(lo).min(hi);
            c2[i] = c2[i].max(lo).min(hi);
        }

        (c1, c2)
    }

    /// Polynomial mutation
    fn polynomial_mutation(&mut self, x: &mut [f64]) {
        let d = x.len();
        for i in 0..d {
            if self.rand() > self.config.mutation_prob {
                continue;
            }

            let (lo, hi) = self.config.bounds[i];
            let y = x[i];
            let delta1 = (y - lo) / (hi - lo);
            let delta2 = (hi - y) / (hi - lo);

            let u = self.rand();
            let deltaq = if u < 0.5 {
                let xy = 1.0 - delta1;
                let val = 2.0 * u + (1.0 - 2.0 * u) * xy.powf(self.config.eta_m + 1.0);
                val.powf(1.0 / (self.config.eta_m + 1.0)) - 1.0
            } else {
                let xy = 1.0 - delta2;
                let val = 2.0 * (1.0 - u) + 2.0 * (u - 0.5) * xy.powf(self.config.eta_m + 1.0);
                1.0 - val.powf(1.0 / (self.config.eta_m + 1.0))
            };

            x[i] = y + deltaq * (hi - lo);
            x[i] = x[i].max(lo).min(hi);
        }
    }

    /// Create offspring population
    pub fn create_offspring<F>(&mut self, eval_fn: &F) -> Vec<Individual>
    where
        F: Fn(&[f64]) -> (Vec<f64>, f64, i32, Vec<f64>),
    {
        let n = self.config.pop_size;
        let mut offspring = Vec::with_capacity(n);

        while offspring.len() < n {
            let p1_idx = self.tournament_select();
            let p2_idx = self.tournament_select();
            let p1 = &self.population[p1_idx].x;
            let p2 = &self.population[p2_idx].x;

            let (mut c1, mut c2) = self.sbx_crossover(p1, p2);
            self.polynomial_mutation(&mut c1);
            self.polynomial_mutation(&mut c2);

            let mut ind1 = Individual::new(c1);
            let (f, cv, status, outputs) = eval_fn(&ind1.x);
            ind1.f = f;
            ind1.cv = cv;
            ind1.status = status;
            ind1.outputs = outputs;
            offspring.push(ind1);

            if offspring.len() < n {
                let mut ind2 = Individual::new(c2);
                let (f, cv, status, outputs) = eval_fn(&ind2.x);
                ind2.f = f;
                ind2.cv = cv;
                ind2.status = status;
                ind2.outputs = outputs;
                offspring.push(ind2);
            }
        }

        offspring
    }

    /// Environmental selection (truncate to pop_size)
    pub fn environmental_selection(&mut self, offspring: Vec<Individual>) {
        // Combine parent and offspring
        let mut combined = self.population.clone();
        combined.extend(offspring);

        // Store combined for sorting
        self.population = combined;
        self.non_dominated_sort();
        self.crowding_distance();

        // Sort by rank, then crowding distance (descending)
        let mut indices: Vec<usize> = (0..self.population.len()).collect();
        indices.sort_by(|&a, &b| {
            let rank_cmp = self.population[a].rank.cmp(&self.population[b].rank);
            if rank_cmp != Ordering::Equal {
                return rank_cmp;
            }
            // Higher crowding distance is better
            self.population[b].crowding_distance
                .partial_cmp(&self.population[a].crowding_distance)
                .unwrap_or(Ordering::Equal)
        });

        // Select top pop_size individuals
        let selected: Vec<Individual> = indices
            .into_iter()
            .take(self.config.pop_size)
            .map(|i| self.population[i].clone())
            .collect();

        self.population = selected;
    }

    /// Run optimization
    pub fn optimize<F>(&mut self, eval_fn: F) -> ParetoFront
    where
        F: Fn(&[f64]) -> (Vec<f64>, f64, i32, Vec<f64>),
    {
        // Initialize
        self.initialize_population();
        self.evaluate(&eval_fn);
        self.non_dominated_sort();
        self.crowding_distance();

        // Main loop
        for _gen in 0..self.config.generations {
            let offspring = self.create_offspring(&eval_fn);
            self.environmental_selection(offspring);
        }

        // Extract Pareto front (rank 0)
        let front: Vec<Individual> = self.population
            .iter()
            .filter(|ind| ind.rank == 0)
            .cloned()
            .collect();

        ParetoFront {
            solutions: front,
            generation: self.config.generations,
            hypervolume: None,
        }
    }

    /// Get current population
    pub fn get_population(&self) -> &[Individual] {
        &self.population
    }
}

/// Compute hypervolume indicator (2D only for simplicity)
pub fn hypervolume_2d(front: &[Individual], ref_point: (f64, f64)) -> f64 {
    if front.is_empty() {
        return 0.0;
    }

    // Sort by first objective
    let mut sorted: Vec<&Individual> = front.iter().collect();
    sorted.sort_by(|a, b| {
        a.f[0].partial_cmp(&b.f[0]).unwrap_or(Ordering::Equal)
    });

    let mut hv = 0.0;
    let mut prev_f2 = ref_point.1;

    for ind in sorted {
        if ind.f[0] < ref_point.0 && ind.f[1] < ref_point.1 {
            let width = ref_point.0 - ind.f[0];
            let height = prev_f2 - ind.f[1];
            if height > 0.0 {
                hv += width * height;
            }
            prev_f2 = ind.f[1];
        }
    }

    hv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dominance() {
        let mut a = Individual::new(vec![1.0, 1.0]);
        a.f = vec![1.0, 2.0];
        a.cv = 0.0;

        let mut b = Individual::new(vec![1.0, 1.0]);
        b.f = vec![2.0, 3.0];
        b.cv = 0.0;

        assert!(a.dominates(&b));
        assert!(!b.dominates(&a));
    }

    #[test]
    fn test_nsga2_simple() {
        let config = NSGA2Config {
            pop_size: 20,
            generations: 5,
            bounds: vec![(0.0, 1.0), (0.0, 1.0)],
            ..Default::default()
        };

        let mut optimizer = NSGA2::new(config);

        // Simple bi-objective test function (ZDT1-like)
        let eval_fn = |x: &[f64]| -> (Vec<f64>, f64, i32, Vec<f64>) {
            let f1 = x[0];
            let g = 1.0 + x[1];
            let f2 = g * (1.0 - (x[0] / g).sqrt());
            (vec![f1, f2], 0.0, 0, vec![])
        };

        let front = optimizer.optimize(eval_fn);
        assert!(!front.solutions.is_empty());
    }
}

use rustc_hash::FxHashMap;
use search_trail::{SaveAndRestore, StateManager};

use crate::logger::Logger;
use crate::branching::BranchingDecision;
use crate::common::*;
use crate::core::components::{ComponentExtractor, ComponentIndex};
use crate::core::problem::Problem;
use crate::preprocess::Preprocessor;
use crate::propagator::Propagator;
use crate::PEAK_ALLOC;
use crate::caching::CacheKey;
use crate::args::Args;
use malachite::Natural;
use std::time::Instant;

pub struct Solver<const S: bool> {
    /// Implication problem of the (Horn) clauses in the input
    problem: Problem,
    /// Manages (save/restore) the states (e.g., reversible primitive types)
    state: StateManager,
    /// Extracts the connected components in the problem
    component_extractor: ComponentExtractor,
    /// Heuristics that decide on which distribution to branch next
    branching_heuristic: Box<dyn BranchingDecision>,
    /// Runs Boolean Unit Propagation and Schlandals' specific propagation at each decision node
    propagator: Propagator,
    cache: FxHashMap<CacheKey, Natural>,
    /// Statistics gathered during the solving
    statistics: Logger<S>,
}

impl<const S: bool> Solver<S> {
    pub fn new(
        problem: Problem,
        state: StateManager,
        component_extractor: ComponentExtractor,
        branching_heuristic: Box<dyn BranchingDecision>,
        propagator: Propagator,
    ) -> Self {
        Self {
            problem,
            state,
            component_extractor,
            branching_heuristic,
            propagator,
            cache: FxHashMap::default(),
            statistics: Logger::default(),
        }
    }

    /// Restores the state of the solver to the previous state
    fn restore(&mut self) {
        self.propagator.restore(&self.state);
        self.state.restore_state();
    }

    /// Solves the problem represented by this solver using a DPLL-search based method.
    pub fn search(&mut self, parameters: &SolverParameters) -> Solution {
        let mut preprocessor = Preprocessor::new(
            &mut self.problem,
            &mut self.state,
            &mut self.propagator,
            &mut self.component_extractor,
        );
        let preproc = preprocessor.preprocess();
        if preproc.is_err() {
            self.statistics.print();
            return Solution::new(natural(0), parameters.start.elapsed().as_secs());
        }
        self.problem.clear_after_preprocess(&mut self.state);
        self.component_extractor.shrink(
            self.problem.number_clauses(),
            self.problem.number_variables(),
        );
        self.propagator.reduce();

        if self.problem.number_clauses() == 0 {
            // TODO
            return Solution::new(natural(0), parameters.start.elapsed().as_secs());
        }
        println!("Launching model count");
        let count = self.model_count(ComponentIndex(0), 1, parameters);
        self.statistics.peak_memory(PEAK_ALLOC.peak_usage_as_mb());
        self.statistics.print();
        Solution::new(count, parameters.start.elapsed().as_secs())
    }

    fn model_count(&mut self, component: ComponentIndex, level: isize, parameters: &SolverParameters) -> Natural {
        if PEAK_ALLOC.current_usage_as_mb() as u64 >= parameters.memory_limit {
            println!("Clearing cache");
            self.cache.clear();
        }
        if parameters.start.elapsed().as_secs() >= parameters.timeout {
            println!("Time out");
            return natural(0);
        }
        let cache_key = self.component_extractor[component].get_cache_key();
        self.statistics.cache_access();

        if let Some(n) = self.cache.get(&cache_key) {
            return n.clone();
        }
        let mut count = natural(0);
        let variable = self.branching_heuristic.branch_on(&self.problem, &mut self.state, &self.component_extractor, component);
        println!("Branching on {}", self.problem[variable]);
        for value in [true, false] {
            self.state.save_state();
            if self.propagator.propagate_variable(variable, value, &mut self.problem, &mut self.state, component, &mut self.component_extractor, level).is_ok() {
                let mut prod = natural(1);
                self.state.save_state();
                if self.component_extractor.detect_components(&mut self.problem, &mut self.state, component) {
                    self.statistics.decomposition(self.component_extractor.number_components(&self.state));
                    for sub_component in self.component_extractor.components_iter(&self.state) {
                        let sub_count = self.model_count(sub_component, level + 1, parameters);
                        prod *= sub_count;
                        if prod == 0 {
                            break;
                        }
                    }
                    count += prod;
                }
                self.restore();
            }
            self.restore();
        }
        self.cache.insert(cache_key, count.clone());
        count
    }
}

pub struct SolverParameters {
    /// Memory limit for the solving, in megabytes. When reached, the cache is cleared. Note that
    /// this parameter should not be used for compilation.
    memory_limit: u64,
    /// Time limit for the search
    timeout: u64,
    /// Time at which the solving started
    start: Instant,
}

impl SolverParameters {

    pub fn new(args: &Args) -> Self {
        Self {
            memory_limit: args.memory(),
            timeout: args.timeout(),
            start: Instant::now(),
        }
    }
}

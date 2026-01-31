// Re-export the modules
mod solver;
mod logger;
pub mod args;
pub mod common;
mod branching;
pub mod core;
mod parser;
mod propagator;
mod preprocess;
mod caching;

use malachite::Natural;

use search_trail::StateManager;

use core::components::ComponentExtractor;
use parser::*;

use propagator::Propagator;
pub use common::*;
use branching::*;
use caching::*;
use args::*;

pub use solver::Solver;
use solver::SolverParameters;

use peak_alloc::PeakAlloc;
#[global_allocator]
pub static PEAK_ALLOC: PeakAlloc = PeakAlloc;

pub fn solve(args: Args) -> Natural {
    let mut state = StateManager::default();
    let propagator = Propagator::new(&mut state);
    let problem = problem_from_cnf(args.input().clone(), &mut state);
    let branching: Box<dyn BranchingDecision> = match args.branching() {
        Branching::First => Box::<First>::default(),
    };
    let caching_scheme = CachingScheme::new(args.caching());
    let component_extractor = ComponentExtractor::new(&problem, caching_scheme, &mut state);
    let parameters = SolverParameters::new(&args);
    if args.statistics() {
        let mut solver = Solver::<true>::new(problem, state, component_extractor, branching, propagator);
        let solution = solver.search(&parameters);
        println!("{}", solution);
        solution.count()
    } else {
        let mut solver = Solver::<false>::new(problem, state, component_extractor, branching, propagator);
        let solution = solver.search(&parameters);
        println!("{}", solution);
        solution.count()
    }
}

use clap::ValueEnum;
use malachite::Natural;

pub fn natural(n: usize) -> Natural {
    Natural::from(n)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Branching {
    First,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Caching {
    Hybrid,
    OmitBinary,
    OmitImplicit,
}

#[derive(Clone)]
pub struct Solution {
    /// Compute count
    count: Natural,
    /// Number of seconds, since the start of the solver, at which the solution was found
    time_found: u64,
}

impl Solution {

    pub fn new(count: Natural, time_found: u64) -> Self {
        Self {
            count,
            time_found,
        }
    }

    pub fn count(&self) -> Natural {
        self.count.clone()
    }

    pub fn print(&self) {
        println!("{}", self);
    }
}

impl std::fmt::Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Model count is {} found in {} seconds", self.count, self.time_found)
    }
}

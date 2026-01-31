use clap::Parser;

use std::path::PathBuf;
use crate::{Branching, Caching};

#[derive(Parser)]
#[clap(name="Crount", version, author, about)]
pub struct Args {
    /// The input file
    #[clap(short, long, value_parser)]
    input: PathBuf,
    /// Stops the search/compilation after timeout seconds
    #[clap(short, long, default_value_t=u64::MAX)]
    timeout: u64,
    /// Distribution selection heuristic
    #[clap(short, long, value_enum, default_value_t=Branching::First)]
    branching: Branching,
    /// Caching strategy
    #[clap(short, long, value_enum, default_value_t=Caching::Hybrid)]
    caching: Caching,
    /// Collect stats during the search
    #[clap(long, action)]
    statistics: bool,
    /// The memory limit, in mega-bytes
    #[clap(short, long, default_value_t=u64::MAX)]
    memory: u64,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            input: PathBuf::default(),
            timeout: u64::MAX,
            branching: Branching::First,
            caching: Caching::Hybrid,
            statistics: false,
            memory: u64::MAX,
        }
    }
}

impl Args {

    pub fn input(&self) -> &PathBuf {
        &self.input
    }

    pub fn timeout(&self) -> u64 {
        self.timeout
    }

    pub fn branching(&self) -> Branching {
        self.branching
    }

    pub fn caching(&self) -> Caching {
        self.caching
    }

    pub fn statistics(&self) -> bool {
        self.statistics
    }

    pub fn memory(&self) -> u64 {
        self.memory
    }

    pub fn set_input(&mut self, value: PathBuf) {
        self.input = value;
    }

    pub fn set_timeout(&mut self, value: u64) {
        self.timeout = value;
    }

    pub fn set_branching(&mut self, value: Branching) {
        self.branching = value;
    }

    pub fn set_caching(&mut self, value: Caching) {
        self.caching = value;
    }


    pub fn set_statistics(&mut self, value: bool) {
        self.statistics = value;
    }

    pub fn set_memory(&mut self, value: u64) {
        self.memory = value;
    }
}

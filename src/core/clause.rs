//! Representation of a clause in Schlandals. All clauses used in Schlandals are Horn clause, which
//! means that they have at most one positive literal, the head of the clause.
//! The literals of the clause (head included) are stored in a vector that implements the 2-watch literals
//! method.
//! However, the specific needs of Schlandals for the propagation impose that each clause is watched by two pairs
//! of watched literals.
//! One pair is composed of deterministic literals, and the other of probabilistic ones.
//! In this way the propagator can, at any time, query a boud on the number of unfixed deterministic/probabilistic
//! variables in the clause.

use search_trail::{BoolManager, ReversibleBool, StateManager};
use super::literal::Literal;
use rustc_hash::FxHashMap;

use super::problem::VariableIndex;

#[derive(Debug)]
pub struct Clause {
    /// id of the clause in the input problem
    id: usize,
    /// The literals of the clause. Implemented using a vector with watched literals
    literals: Vec<Literal>,
    /// Has the clause been learned during the search
    is_learned: bool,
    /// Is the clause active (i.e., not yet satisfied)
    active: ReversibleBool,
    /// Number of deterministic variables in the body of the clause
    is_binary: bool,
    modified: ReversibleBool,
}

impl Clause {

    pub fn new(id: usize, literals: Vec<Literal>, is_learned: bool, state: &mut StateManager) -> Self {
        let is_binary = literals.len() == 2;
        Self {
            id,
            literals,
            is_learned,
            active: state.manage_bool(true),
            is_binary,
            modified: state.manage_bool(false),
        }
    }
    
    /// Set the clause as unconstrained. This operation is reverted when the state manager restore its state.
    pub fn deactivate(&self, state: &mut StateManager) {
        state.set_bool(self.active, false);
    }
    
    /// Returns true iff the clause is constrained
    pub fn is_active(&self, state: &StateManager) -> bool {
        state.get_bool(self.active)
    }
    
    /// Notify the clause that the given variable has taken the given value. Updates the watchers accordingly.
    pub fn notify_variable_value(&mut self, variable: VariableIndex, state: &mut StateManager) -> VariableIndex {
        if self.literals[0].to_variable() == variable {
            self.literals.swap(0, 1);
        }
        for i in 2..self.literals.len() {
            if !self.literals[i].is_variable_fixed(state) {
                self.literals.swap(1, i);
                break;
            }
        }
        self.literals[1].to_variable()
    }

    /// Returns true iff the clause is unit
    pub fn is_unit(&self, state: &StateManager) -> bool {
        if !self.is_active(state) {
            return false;
        }
        if self.literals.len() == 1 {
            return true;
        }
        !self.literals[0].is_variable_fixed(state) && self.literals[1].is_variable_fixed(state)
    }

    /// Returns the last unfixed literal in the unit clause
    pub fn get_unit_assigment(&self, state: &StateManager) -> Literal {
        debug_assert!(self.is_unit(state));
        self.literals[0]
    }

    /// Returns true iff the clause is learned
    pub fn is_learned(&self) -> bool {
        self.is_learned
    }
    
    // --- ITERATORRS --- //

    /// Returns an interator on the literals of the clause
    pub fn iter(&self) -> impl Iterator<Item = Literal> + use<'_> {
        self.literals.iter().copied()
    }
    
    /// Returns an iterator on the variables represented by the literals of the clause
    pub fn iter_variables(&self) -> impl Iterator<Item = VariableIndex> + '_ {
        self.literals.iter().map(|l| l.to_variable())
    }
    
    pub fn clear_literals(&mut self, map: &FxHashMap<VariableIndex, VariableIndex>) {
        for i in (0..self.literals.len()).rev() {
            let v = self.literals[i].to_variable();
            match map.get(&v).copied() {
                Some(new_v) => {
                    self.literals[i].update_variable(new_v);
                },
                None => {
                    self.literals.swap_remove(i);
                }
            }
        }
    }

    pub fn get_watchers(&self) -> Vec<VariableIndex> {
        self.literals.iter().take(2).map(|l| l.to_variable()).collect()
    }

    pub fn modified(&self, state: &mut StateManager) {
        state.set_bool(self.modified, true);
    }

    pub fn is_modified(&self, state: &StateManager) -> bool {
        state.get_bool(self.modified)
    }

    pub fn is_binary(&self) -> bool {
        self.is_binary
    }
    
}

// Writes a clause as C{id}: l1 l2 ... ln
impl std::fmt::Display for Clause {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "C{}: {}", self.id + 1, self.literals.iter().map(|l| format!("{}", l)).collect::<Vec<String>>().join(" "))
    }
}

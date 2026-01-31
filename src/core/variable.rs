use search_trail::*;
use crate::core::problem::ClauseIndex;
use rustc_hash::FxHashMap;
use super::sparse_set::SparseSet;

/// Data structure that actually holds the data of a  variable of the input problem
#[derive(Debug)]
pub struct Variable {
    /// The id of the variable in the input problem
    id: usize,
    /// The clauses in which the variable appears with positive polarity
    clauses_positive: SparseSet<ClauseIndex>,
    /// The clauses in which the variable appears with negative polarity
    clauses_negative: SparseSet<ClauseIndex>,
    /// The value assigned to the variable
    value: ReversibleOptionBool,
    /// Level at which the decision was made for this variable
    decision: isize,
    /// Index in the assignment stack at which the decision has been made for the variable
    assignment_position: ReversibleUsize,
}

impl Variable {
    
    pub fn new(id: usize, state: &mut StateManager) -> Self {
        Self {
            id,
            clauses_positive: SparseSet::new(state),
            clauses_negative: SparseSet::new(state),
            value: state.manage_option_bool(None),
            decision: -1,
            assignment_position: state.manage_usize(0),
        }
    }

    /// Sets the variable to the given value. This operation is reverted when
    /// the trail is restored
    pub fn set_value(&self, value: bool, state: &mut StateManager) {
        state.set_option_bool(self.value, value);
    }
    
    /// Returns the value of the variable
    pub fn value(&self, state: &StateManager) -> Option<bool> {
        state.get_option_bool(self.value)
    }
    
    /// Returns the reversible boolean representing the value assignment
    /// of the variable
    pub fn get_value_index(&self) -> ReversibleOptionBool {
        self.value
    }
    
    /// Returns true iff the variable is fixed
    pub fn is_fixed(&self, state: &StateManager) -> bool {
        state.get_option_bool(self.value).is_some()
    }
    
    /// Adds the clause in the positive occurence list
    pub fn add_clause_positive_occurence(&mut self, clause: ClauseIndex, state: &mut StateManager) {
        self.clauses_positive.add(clause, state);
    }
    
    /// Adds the clause in the negative occurence list
    pub fn add_clause_negative_occurence(&mut self, clause: ClauseIndex, state: &mut StateManager) {
        self.clauses_negative.add(clause, state);
    }
    
    /// Sets the decision level for the variable to the given level
    pub fn set_decision_level(&mut self, level: isize) {
        self.decision = level
    }
    
    /// Returns the decision level for the variable. This function assume that the query is done
    /// only on fixed variable since the level is not reversible. Since this function is used in
    /// clause learning, it should always be the case
    pub fn decision_level(&self) -> isize {
        self.decision
    }

    /// Sets the assignment position (in the assignment stack) of the variable to the given value
    pub fn set_assignment_position(&self, position: usize, state: &mut StateManager) {
        state.set_usize(self.assignment_position, position);
    }
    
    /// Returns the assignment position (in the assignment stack) of the variable
    pub fn get_assignment_position(&self, state: &StateManager) -> usize {
        state.get_usize(self.assignment_position)
    }

    pub fn number_clauses(&self, state: &StateManager) -> usize {
        self.clauses_positive.len(state) + self.clauses_negative.len(state)
    }
    
    // --- ITERATOR --- //

    /// Returns an iterator on the clauses in which the variable appears with a positive polarity
    pub fn iter_clauses_positive_occurence(&self, state: &StateManager) -> impl Iterator<Item = ClauseIndex> + '_ {
        self.clauses_positive.iter(state)
    }
    
    /// Returns an iterator on the clauses in which the variable appears with a negative polarity
    pub fn iter_clauses_negative_occurence(&self, state: &StateManager) -> impl Iterator<Item = ClauseIndex> + '_ {
        self.clauses_negative.iter(state)
    }

    pub fn clear_clauses(&mut self, map: &FxHashMap<ClauseIndex, ClauseIndex>, state: &mut StateManager) -> usize {
        self.clauses_positive.clear(map, state);
        self.clauses_negative.clear(map, state);
        self.clauses_positive.len(state) + self.clauses_negative.len(state)
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{}", self.id + 1)
    }
}

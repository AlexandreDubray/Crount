use search_trail::{StateManager, UsizeManager, ReversibleUsize};

use crate::core::components::{ComponentIndex, ComponentExtractor};
use crate::core::problem::{Problem, VariableIndex};

use super::core::literal::Literal;

pub type PropagationResult = Result<(), isize>;

pub struct Propagator {
    propagation_stack: Vec<(VariableIndex, bool, isize)>,
    assignments: Vec<Literal>,
    base_assignments: ReversibleUsize,
}

impl Propagator {
    
    pub fn new(state: &mut StateManager) -> Self {
        Self {
            propagation_stack: vec![],
            assignments: vec![],
            base_assignments: state.manage_usize(0),
        }
    }
    
    /// Adds a variable to be propagated with the given value
    pub fn add_to_propagation_stack(&mut self, variable: VariableIndex, value: bool, level: isize) {
        self.propagation_stack.push((variable, value, level));
    }
    
    /// Propagates a variable to the given value. The component of the variable is also given to be able to use the {f-t}-reachability.
    pub fn propagate_variable(&mut self, variable: VariableIndex, value: bool, g: &mut Problem, state: &mut StateManager, component: ComponentIndex, extractor: &mut ComponentExtractor, level: isize) -> PropagationResult {
        self.add_to_propagation_stack(variable, value, level);
        self.propagate(g, state, component, extractor, level)
    }

    /// Returns an iterator over the assignments made during the last propagation
    pub fn assignments_iter(&self, state: &StateManager) -> impl Iterator<Item = Literal> + '_{
        let start = state.get_usize(self.base_assignments);
        self.assignments.iter().skip(start).copied()
    }

    /// Returns true if there are any assignments in the assignments queue
    pub fn has_assignments(&self, state: &StateManager) -> bool {
        let start = state.get_usize(self.base_assignments);
        start < self.assignments.len()
    }
    
    /// Clears the propagation stack as well as the unconstrained clauses stack. This function
    /// is called when an UNSAT has been encountered.
    fn clear(&mut self) {
        self.propagation_stack.clear();
    }
    
    pub fn restore(&mut self, state: &StateManager) {
        let limit = state.get_usize(self.base_assignments);
        self.assignments.truncate(limit);
    }

    pub fn propagate(&mut self, g: &mut Problem, state: &mut StateManager, component: ComponentIndex, extractor: &mut ComponentExtractor, level: isize) -> PropagationResult {
        state.set_usize(self.base_assignments, self.assignments.len());
        while let Some((variable, value, l)) = self.propagation_stack.pop() {
            if let Some(v) = g[variable].value(state) {
                if v == value {
                    continue;
                }
                self.clear();
                return PropagationResult::Err(level);
            }
            g[variable].set_assignment_position(self.assignments.len(), state);
            self.assignments.push(Literal::from_variable(variable, value, g[variable].get_value_index()));
            g.set_variable(variable, value, l, state);
            
            if value {
                for clause in g[variable].iter_clauses_positive_occurence(state) {
                    g[clause].deactivate(state);
                }
            } else {
                for clause in g[variable].iter_clauses_negative_occurence(state) {
                    g[clause].deactivate(state);
                }
            }

            for i in (0..g.number_watchers(variable)).rev() {
                let clause = g.get_clause_watched(variable, i);
                if g[clause].is_active(state) {
                    g[clause].modified(state);
                    let new_watcher = g[clause].notify_variable_value(variable, state);
                    if new_watcher != variable {
                        g.remove_watcher(variable, i);
                        g.add_watcher(new_watcher, clause);
                    }
                    if g[clause].is_unit(state) {
                        let l = g[clause].get_unit_assigment(state);
                        self.add_to_propagation_stack(l.to_variable(), l.is_positive(), level);
                    }
                }
            }
        }
        PropagationResult::Ok(())
    }

    pub fn iter_propagated_assignments(&self) -> impl Iterator<Item = Literal> + '_ {
        self.assignments.iter().copied()
    }

    pub fn reduce(&mut self) {
        self.assignments.clear();
    }
}

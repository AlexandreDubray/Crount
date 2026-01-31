use search_trail::*;
use super::literal::*;
use super::variable::*;
use super::clause::*;

use rustc_hash::FxHashMap;

/// Abstraction used as a typesafe way of retrieving a `Variable` in the `Problem` structure
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VariableIndex(pub usize);

/// Abstraction used as a typesafe way of retrieving a `Clause` in the `Problem` structure
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ClauseIndex(pub usize);

/// Data structure representing the Problem.
#[derive(Debug)]
pub struct Problem {
    /// Vector containing the nodes of the problem
    variables: Vec<Variable>,
    /// Vector containing the clauses of the problem
    clauses: Vec<Clause>,
    /// Store for each variables the clauses it watches
    watchers: Vec<Vec<ClauseIndex>>,
    /// Number of clauses in the problem
    number_clauses_problem: usize,
}

impl Problem {
    
    // --- PROBLEM CREATION --- //

    /// Creates a new empty implication problem
    pub fn new(state: &mut StateManager, n_var: usize, n_clause: usize) -> Self {
        let variables = (0..n_var).map(|i| Variable::new(i, state)).collect();
        let watchers = (0..n_var).map(|_| vec![]).collect();
        Self {
            variables,
            clauses: vec![],
            watchers,
            number_clauses_problem: n_clause,
        }
    }

    pub fn add_clause(
        &mut self,
        literals: Vec<Literal>,
        state: &mut StateManager,
        is_learned: bool,
    ) -> ClauseIndex {
        let cid = ClauseIndex(self.clauses.len());
        for variable in literals.iter().take(2).map(|l| l.to_variable()) {
            self.watchers[variable.0].push(cid);
        }

        let clause = Clause::new(cid.0, literals, is_learned, state);

        for literal in clause.iter() {
            let variable = literal.to_variable();
            if literal.is_positive() {
                self[variable].add_clause_positive_occurence(cid, state);
            } else {
                self[variable].add_clause_negative_occurence(cid, state);
            }
        }
        self.clauses.push(clause);
        cid
    }


    /// Clears unnecessary space for the problem. After pre-processing, some variables, clauses or
    /// distributions might not be used anymore, we can remove them from the representation.
    pub fn clear_after_preprocess(&mut self, state: &mut StateManager) {
        // First, we delete the clauses 
        let mut clauses_map: FxHashMap<ClauseIndex, ClauseIndex> = FxHashMap::default();
        let mut new_clause_index = 0;
        for (i, clause) in self.clauses_iter().enumerate() {
            if self[clause].is_active(state) {
                clauses_map.insert(clause, ClauseIndex(new_clause_index));
                self.clauses.swap(i, new_clause_index);
                new_clause_index += 1;
            }
        }
        self.clauses.truncate(new_clause_index);
        self.clauses.shrink_to_fit();

        if self.clauses.is_empty() {
            return;
        }

        self.number_clauses_problem = self.clauses.len();

        let mut variables_map: FxHashMap<VariableIndex, VariableIndex> = FxHashMap::default();
        let mut new_variable_index = 0;
        for (i, variable) in self.variables_iter().enumerate() {
            self.watchers[i].clear();
            let number_remaining_clauses = self[variable].clear_clauses(&clauses_map, state);
            // Variable is not in the problem anymore
            if number_remaining_clauses == 0 || self[variable].is_fixed(state) {
                continue;
            }
            variables_map.insert(variable, VariableIndex(new_variable_index));
            self.variables.swap(i, new_variable_index);
            self.watchers.swap(i, new_variable_index);
            new_variable_index += 1;
        }

        self.variables.truncate(new_variable_index);
        self.variables.shrink_to_fit();
        self.watchers.truncate(new_variable_index);
        self.watchers.shrink_to_fit();

        for clause in self.clauses_iter() {
            self[clause].clear_literals(&variables_map);
            for v in self[clause].get_watchers() {
                self.watchers[v.0].push(clause);
            }
        }
    }
    
    // --- problem MODIFICATIONS --- //
    
    /// Sets a variable to true or false.
    ///     - If true, Removes the variable from the body of the constrained clauses
    ///     - If false, and probabilistic, increase the counter of false variable in the distribution
    /// If the variable is the min or max variable not fixed, update the boundaries accordingly.
    pub fn set_variable(&mut self, variable: VariableIndex, value: bool, level: isize, state: &mut StateManager) {
        self[variable].set_value(value, state);
        self[variable].set_decision_level(level);
    }

    /// Returns the number of clauses watched by the variable
    pub fn number_watchers(&self, variable: VariableIndex) -> usize {
        self.watchers[variable.0].len()
    }

    /// Returns the clause watched by the variable at id watcher_id
    pub fn get_clause_watched(&self, variable: VariableIndex, watcher_id: usize) -> ClauseIndex {
        self.watchers[variable.0][watcher_id]
    }
    
    pub fn remove_watcher(&mut self, variable: VariableIndex, watcher_id: usize) {
        self.watchers[variable.0].swap_remove(watcher_id);
    }
    
    pub fn add_watcher(&mut self, variable: VariableIndex, clause: ClauseIndex) {
        self.watchers[variable.0].push(clause);
    }

    // --- QUERIES --- //
    
    /// Set a clause as unconstrained
    pub fn deactivate_clause(&self, clause: ClauseIndex, state: &mut StateManager) {
        self[clause].deactivate(state);
    }
    
    // --- GETTERS --- //
    
    /// Returns the number of clause in the problem
    pub fn number_clauses(&self) -> usize {
        self.clauses.len()
    }

    pub fn last_clause_subproblem(&self) -> ClauseIndex {
        ClauseIndex(self.clauses.len() - 1)
    }

    /// Returns the number of unlearned clauses (i.e., the number of clauses in the initial problem
    pub fn number_clauses_problem(&self) -> usize {
        self.number_clauses_problem
    }

    /// Returns the number of variable in the problem
    pub fn number_variables(&self) -> usize {
        self.variables.len()
    }
    
    // --- ITERATORS --- //
    
    /// Returns an iterator on all (constrained and unconstrained) the clauses of the problem
    pub fn clauses_iter(&self) -> impl Iterator<Item = ClauseIndex> + use<> {
        (0..self.clauses.len()).map(ClauseIndex)
    }

    pub fn variables_iter(&self) -> impl Iterator<Item = VariableIndex> + use<> {
        (0..self.variables.len()).map(VariableIndex)
    }
}

// --- Indexing the problem with the various indexes --- //

impl std::ops::Index<VariableIndex> for Problem {
    type Output = Variable;

    fn index(&self, index: VariableIndex) -> &Self::Output {
        &self.variables[index.0]
    }
}

impl std::ops::IndexMut<VariableIndex> for Problem {
    fn index_mut(&mut self, index: VariableIndex) -> &mut Self::Output {
        &mut self.variables[index.0]
    }
}

impl std::ops::Index<ClauseIndex> for Problem {
    type Output = Clause;

    fn index(&self, index: ClauseIndex) -> &Self::Output {
        &self.clauses[index.0]
    }
}

impl std::ops::IndexMut<ClauseIndex> for Problem {
    fn index_mut(&mut self, index: ClauseIndex) -> &mut Self::Output {
        &mut self.clauses[index.0]
    }
}

// --- Operator on the indexes for the vectors --- //

impl std::ops::Add<usize> for VariableIndex {
    type Output = VariableIndex;

    fn add(self, rhs: usize) -> Self::Output {
        VariableIndex(self.0 + rhs)   
    }
}

impl std::ops::AddAssign<usize> for VariableIndex {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl std::ops::Sub<usize> for VariableIndex {
    type Output = VariableIndex;

    fn sub(self, rhs: usize) -> Self::Output {
        VariableIndex(self.0 - rhs)
    }
}

impl std::ops::SubAssign<usize> for VariableIndex {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

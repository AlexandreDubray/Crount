use search_trail::StateManager;
use crate::core::components::{ComponentExtractor, ComponentIndex};
use crate::core::problem::{VariableIndex, Problem};

pub trait BranchingDecision {
    fn branch_on(&mut self, p: &Problem, state: &StateManager, component_extractor: &ComponentExtractor, component: ComponentIndex) -> VariableIndex;
}

#[derive(Default)]
pub struct First {}

impl BranchingDecision for First {
    fn branch_on(&mut self, p: &Problem, state: &StateManager, component_extractor: &ComponentExtractor, component: ComponentIndex) -> VariableIndex {
        for clause in component_extractor.component_iter(component) {
            if p[clause].is_active(state) {
                return p[clause].iter_variables().find(|v| !p[*v].is_fixed(state)).unwrap();
            }
        }
        panic!("Could not find unassigned variables during branching");
    }
}

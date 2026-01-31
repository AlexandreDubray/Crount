use crate::core::problem::{VariableIndex, Problem};
use crate::core::literal::Literal;
use search_trail::StateManager;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn problem_from_cnf(input: PathBuf, state: &mut StateManager) -> Problem {
    let mut clauses: Vec<Vec<isize>> = vec![];
    let file = File::open(input).unwrap();
    let reader = BufReader::new(file);
    for l in reader.lines() {
        match l {
            Err(e) => panic!("Problem while reading file: {}", e),
            Ok(line) => {
                if !line.starts_with('c') && !line.starts_with('p') {
                    // Note: the space before the 0 is important so that clauses like "1 -10 0" are correctly splitted
                    for clause in line.trim_end().split(" 0").filter(|cl| !cl.is_empty()) {
                        clauses.push(clause.split_whitespace().map(|x| x.parse::<isize>().unwrap()).collect());
                    }
                }
            }
        }
    }

    let mut number_var = 0;
    for clause in clauses.iter() {
        number_var = number_var.max(clause.iter().map(|l| l.unsigned_abs()).max().unwrap());
    }
    let mut problem = Problem::new(state, number_var, clauses.len());
    for clause in clauses.iter() {
        let mut literals: Vec<Literal> = vec![];
        for lit in clause.iter().copied() {
            if lit == 0 {
                panic!("Variables in clauses can not be 0");
            }
            let variable = lit.unsigned_abs();
            let var = VariableIndex(variable - 1);
            let trail_value_index = problem[var].get_value_index();
            let literal = Literal::from_variable(var, lit > 0, trail_value_index);
            literals.push(literal);
        }
        problem.add_clause(literals, state, false);
    }
    problem
}

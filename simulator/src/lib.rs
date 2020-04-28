#[macro_use]
extern crate lazy_static;

use crate::dto::{python, rust};
use crate::runner::*;
use crate::sim::Simulation;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use serde_json;

pub mod data;
pub mod dto;
pub mod runner;
pub mod sim;

#[pyclass(module = "simulator")]
struct Patch {
    pub patch: rust::Patch,
}

#[pymethods]
impl Patch {
    #[new]
    fn new(patch_json: &str) -> Self {
        // TODO: Exception handling
        let py_patch: python::Patch = serde_json::from_str(&patch_json).unwrap();
        Patch {
            patch: rust::Patch::from_python(py_patch),
        }
    }
}

#[pyclass(module = "simulator")]
struct Arena {
    pub arena: python::Arena,
}

#[pymethods]
impl Arena {
    #[new]
    fn new(arena_json: &str) -> Self {
        // TODO: Exception handling
        let py_arena: python::Arena = serde_json::from_str(&arena_json).unwrap();
        Arena { arena: py_arena }
    }
}

#[pyfunction]
fn run_simulation(patch: &Patch, arena: &Arena, match_up: &str, num_runs: i32) -> f64 {
    // TODO: Exception handling
    let py_match_up: python::MatchUp = serde_json::from_str(&match_up).unwrap();
    let match_up = rust::MatchUp::from_python(py_match_up, arena.arena.clone());
    let arena = sim::Arena::from_dto(match_up.arena.clone());
    let combatant_infos = match_to_combatant_infos(&patch.patch, &match_up);
    let combatants = match_to_combatants(&combatant_infos);
    let (left_wins_percent, _time_outs) = run_many_sims(num_runs, &combatants, &arena);
    left_wins_percent
}

#[pymodule]
fn simulator(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Patch>()?;
    m.add_class::<Arena>()?;
    m.add_wrapped(wrap_pyfunction!(run_simulation))?;
    Ok(())
}

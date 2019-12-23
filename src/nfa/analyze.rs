use std::collections::HashSet;
use std::iter::FromIterator;

use super::{CharRange, Transition, NFA};

pub fn starting_chars(nfa: &NFA) -> Vec<CharRange> {
    let mut state = nfa.execution_state();
    nfa.initialize_states(&mut state.current);

    let mut ranges = HashSet::new();

    for i in state.current.iter() {
        let s = &nfa.states[i];
        for t in s.transitions.iter() {
            match t {
                Transition::Ranges(r, _) => {
                    for (low, high) in r.iter() {
                        ranges.insert((*low, *high));
                    }
                }
            }
        }
    }

    Vec::from_iter(ranges)
}

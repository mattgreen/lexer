use std::char;

use hashbrown::HashSet;

use super::{Transition, NFA};

pub fn starting_chars(nfa: &NFA) -> HashSet<char> {
    let mut state = nfa.execution_state();
    nfa.initialize_states(&mut state.current);

    let mut chars = HashSet::new();

    for i in state.current.ones() {
        let s = &nfa.states[i];
        for t in s.transitions.iter() {
            match t {
                Transition::Ranges(r, _) => {
                    for (low, high) in r.iter() {
                        for i in (*low as u32)..=(*high as u32) {
                            if let Some(c) = char::from_u32(i) {
                                chars.insert(c);
                            }
                        }
                    }
                }
            }
        }
    }

    chars
}

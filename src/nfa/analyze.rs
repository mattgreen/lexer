use std::char;

use hashbrown::HashSet;

use super::NFA;

impl NFA {
    pub fn starting_chars(&self) -> HashSet<char> {
        let mut state = self.execution_state();
        self.initialize_states(&mut state.current);

        let mut chars = HashSet::new();

        for i in state.current.ones() {
            let s = &self.states[i];
            for t in s.transitions.iter() {
                for (low, high) in t.ranges.iter() {
                    for i in (*low as u32)..=(*high as u32) {
                        if let Some(c) = char::from_u32(i) {
                            chars.insert(c);
                        }
                    }
                }
            }
        }

        chars
    }
}

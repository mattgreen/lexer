use bit_set::BitSet;

#[derive(Clone, Debug)]
pub struct NFA {
    states: Vec<State>,
}

#[derive(Clone, Debug)]
pub struct State {
    accept: bool,
    transitions: Vec<Transition>,
    epsilon_transitions: Vec<StateID>,
}

#[derive(Clone, Debug)]
pub enum Transition {
    Ranges(Vec<CharRange>, StateID),
    Set(String, StateID),
}

pub struct ExecutionState {
    pub current: States,
    pub next: States,
}

pub type States = BitSet;
pub type CharRange = (char, char);
type StateID = usize;

impl NFA {
    pub fn new(states: Vec<State>) -> NFA {
        NFA { states }
    }

    pub fn has_match_state(&self, states: &States) -> bool {
        states.iter().any(|i| self.states[i].accept)
    }

    pub fn is_dead_state(&self, states: &States) -> bool {
        states.is_empty()
    }

    pub fn execution_state(&self) -> ExecutionState {
        let current = self.states();
        let next = self.states();

        ExecutionState { current, next }
    }

    pub fn states(&self) -> States {
        let mut states = BitSet::with_capacity(self.states.len());
        states.insert(0);
        states
    }

    pub fn initialize_states(&self, states: &mut States) {
        states.clear();
        self.add_states(states, 0);
    }

    pub fn longest_match(&self, input: &str, state: &mut ExecutionState) -> Option<usize> {
        state.current.clear();
        self.add_states(&mut state.current, 0);

        if input.is_empty() {
            return if self.has_match_state(&state.current) {
                Some(0)
            } else {
                None
            };
        }

        let mut match_len = None;

        for (len, c) in input.bytes().enumerate() {
            self.step(&state.current, c as char, &mut state.next);

            if self.has_match_state(&state.next) {
                match_len = Some(len + 1);
            } else if self.is_dead_state(&state.next) {
                break;
            }

            std::mem::swap(&mut state.current, &mut state.next);
        }

        match_len
    }

    pub fn step(&self, current: &States, c: char, next: &mut States) {
        next.clear();

        for i in current.iter() {
            let state = &self.states[i];

            if let Some(to) = state.transition_for(c) {
                self.add_states(next, to);
            }
        }
    }

    fn add_states(&self, states: &mut States, idx: StateID) {
        let state = &self.states[idx as usize];
        if state.accept || !state.transitions.is_empty() {
            states.insert(idx as usize);
        }

        for epsilon_idx in state.epsilon_transitions.iter() {
            self.add_states(states, *epsilon_idx);
        }
    }
}

impl State {
    pub fn new(transitions: &[Transition], epsilon_transitions: &[StateID]) -> State {
        State {
            accept: false,
            transitions: transitions.to_vec(),
            epsilon_transitions: epsilon_transitions.to_vec(),
        }
    }

    pub fn accept(transitions: &[Transition], epsilon_transitions: &[StateID]) -> State {
        State {
            accept: true,
            transitions: transitions.to_vec(),
            epsilon_transitions: epsilon_transitions.to_vec(),
        }
    }

    pub fn patch(&mut self, from: StateID, to: StateID) {
        for t in self.transitions.iter_mut() {
            match t {
                Transition::Ranges(_, target) => {
                    if *target == from {
                        *target = to;
                    }
                }
                Transition::Set(_, target) => {
                    if *target == from {
                        *target = to;
                    }
                }
            }
        }

        for t in self.epsilon_transitions.iter_mut() {
            if *t == from {
                *t = to;
            }
        }
    }

    fn transition_for(&self, c: char) -> Option<StateID> {
        self.transitions
            .iter()
            .filter_map(|t| {
                match t {
                    Transition::Ranges(r, to) => {
                        if r.iter().any(|(l, h)| c >= *l && c <= *h) {
                            return Some(*to);
                        }
                    }
                    Transition::Set(s, to) => {
                        if s.find(c).is_some() {
                            return Some(*to);
                        }
                    }
                }

                None
            })
            .nth(0)
    }
}

impl Transition {
    pub fn ranges(sets: &[CharRange], to: StateID) -> Transition {
        Transition::Ranges(sets.to_owned(), to)
    }

    pub fn set(set: &str, to: StateID) -> Transition {
        Transition::Set(set.to_owned(), to)
    }
}

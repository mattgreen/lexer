use regex_syntax::hir::{self, Hir, HirKind};
use regex_syntax::Parser;

use crate::nfa::{State, Transition, NFA};

pub enum Error {
    InvalidPattern,
}

pub fn compile(pattern: &str) -> NFA {
    let hir = Parser::new().parse(pattern).unwrap();
    let mut states = vec![];
    compile_hir(&hir, &mut states);

    states.push(State::accept(&[], &[]));

    NFA::new(states)
}

fn compile_hir(hir: &Hir, states: &mut Vec<State>) {
    match hir.kind() {
        HirKind::Alternation(alternatives) => {
            let mut fixups = vec![];
            for (i, alt) in alternatives.iter().enumerate() {
                let needs_branch = i < (alternatives.len() - 1);

                let start = states.len();
                if needs_branch {
                    states.push(State::new(&[], &[start + 1, 0]));
                }

                compile_hir(&alt, states);
                fixups.push((states.len() - 1, states.len()));

                if needs_branch {
                    states[start] = State::new(&[], &[start + 1, states.len()]);
                }
            }

            let end = states.len();
            for (i, target) in fixups {
                let state = &mut states[i];
                state.patch(target, end);
            }
        }
        HirKind::Anchor(_) => panic!("anchor assertions are not supported"),
        HirKind::Class(class) => {
            match class {
                hir::Class::Unicode(unicode) => {
                    let ranges = unicode.iter()
                        .map(|r| (r.start(), r.end()))
                        .collect::<Vec<_>>();

                    states.push(State::new(
                        &[Transition::ranges(&ranges, states.len() + 1)],
                        &[],
                    ));

                }
                hir::Class::Bytes(_) => unimplemented!("byte classes"),
            }
        }
        HirKind::Concat(children) => {
            for c in children.iter() {
                compile_hir(&c, states);
            }
        }
        HirKind::Empty => {}
        HirKind::Group(group) => {
            compile_hir(&group.hir, states);
        }
        HirKind::Literal(hir::Literal::Unicode(c)) => {
            states.push(State::new(
                &[Transition::ranges(&[(*c, *c)], states.len() + 1)],
                &[],
            ));
        }
        HirKind::Literal(hir::Literal::Byte(_)) => unimplemented!("byte literals"),
        HirKind::Repetition(rep) => {
            if !rep.greedy {
                panic!("non-greedy repetitions are not supported");
            }

            let start = states.len();
            match rep.kind {
                hir::RepetitionKind::ZeroOrOne => {
                    states.push(State::new(&[], &[0, 0]));
                    compile_hir(&rep.hir, states);
                    states[start] = State::new(&[], &[start + 1, states.len()]);
                }
                hir::RepetitionKind::ZeroOrMore => {
                    states.push(State::new(&[], &[0, 0]));
                    compile_hir(&rep.hir, states);
                    states.push(State::new(&[], &[start + 1, states.len() + 1]));
                    states[start] = State::new(&[], &[start + 1, states.len()]);
                }
                hir::RepetitionKind::OneOrMore => {
                    compile_hir(&rep.hir, states);
                    states.push(State::new(&[], &[start, states.len() + 1]));
                }
                hir::RepetitionKind::Range(_) => unimplemented!("repetition range"),
            }
        }
        HirKind::WordBoundary(_) => panic!("word boundaries are not supported"),
    }
}

mod tests {
    use super::compile;
    use crate::nfa::NFA;

    fn matches(nfa: &NFA, s: &str) -> bool {
        let mut state = nfa.execution_state();
        nfa.initialize_states(&mut state.current);

        nfa.longest_match(s, &mut state) == Some(s.len())
    }

    #[test]
    fn a() {
        let nfa = compile("a");

        assert_eq!(matches(&nfa, "a"), true);
        assert_eq!(matches(&nfa, ""), false);
        assert_eq!(matches(&nfa, "bbbb"), false);
    }

    #[test]
    fn aa() {
        let nfa = compile("aa");

        assert_eq!(matches(&nfa, "a"), false);
        assert_eq!(matches(&nfa, "aa"), true);
    }

    #[test]
    fn a_rep() {
        let nfa = compile("a+");

        assert_eq!(matches(&nfa, "aaaaaaa"), true);
    }

    #[test]
    fn a_zero_or_one() {
        let nfa = compile("a?");

        assert_eq!(matches(&nfa, ""), true);
        assert_eq!(matches(&nfa, "a"), true);
        assert_eq!(matches(&nfa, "ab"), false);
        assert_eq!(matches(&nfa, "bb"), false);
    }

    #[test]
    fn a_zero_or_more() {
        let nfa = compile("a*");

        assert_eq!(matches(&nfa, ""), true);
        assert_eq!(matches(&nfa, "a"), true);
        assert_eq!(matches(&nfa, "aaaaaaa"), true);
        assert_eq!(matches(&nfa, "b"), false);
    }

    #[test]
    fn classes() {
        let nfa = compile("[a-zA-Z]");

        assert_eq!(matches(&nfa, "a"), true);
        assert_eq!(matches(&nfa, "0"), false);
        assert_eq!(matches(&nfa, ""), false);
    }

    #[test]
    fn class_rep() {
        let nfa = compile("[a-zA-Z]+");

        assert_eq!(matches(&nfa, "a"), true);
        assert_eq!(matches(&nfa, "aaaaaa"), true);
        assert_eq!(matches(&nfa, "aa0"), false);
        assert_eq!(matches(&nfa, ""), false);
    }

    #[test]
    fn class_set() {
        let nfa = compile("[.+'\"]");

        assert_eq!(matches(&nfa, "."), true);
        assert_eq!(matches(&nfa, "+"), true);
        assert_eq!(matches(&nfa, "'"), true);
        assert_eq!(matches(&nfa, "\""), true);
        assert_eq!(matches(&nfa, "a"), false);
        assert_eq!(matches(&nfa, ""), false);
    }

    #[test]
    fn group() {
        let nfa = compile("(ab)a");

        assert_eq!(matches(&nfa, "aba"), true);
        assert_eq!(matches(&nfa, "ab"), false);
        assert_eq!(matches(&nfa, "abaa"), false);
    }

    #[test]
    fn alt() {
        let nfa = compile("aa|bb");

        assert_eq!(matches(&nfa, "aa"), true);
        assert_eq!(matches(&nfa, "bb"), true);
        assert_eq!(matches(&nfa, "cc"), false);
    }
}

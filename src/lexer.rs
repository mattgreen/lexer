use bit_set::BitSet;

use crate::Lexicon;
use crate::nfa::{NFA, ExecutionState};

pub struct Lexer<'input> {
    input: &'input str,
    offset: usize,
    rules: Vec<Rule>,
    matches: Vec<Option<usize>>,
    ignore_chars: [bool; 256],
    prefixes: Vec<BitSet>,
}

#[derive(Debug, PartialEq)]
pub enum Next<'input> {
    Token(usize, &'input str),
    End,
    Error(Error<'input>),
}

#[derive(Debug, PartialEq)]
pub enum Error<'input> {
    UnexpectedChar(&'input str),
}

struct Rule {
    id: usize,
    nfa: NFA,
    state: ExecutionState,
}

impl<'input> Lexer<'input> {
    pub fn new(lexicon: &Lexicon, input: &'input str) -> Self {
        let mut rules = lexicon.rules
            .iter()
            .map(|r| {
                Rule {
                    id: r.id,
                    nfa: r.nfa.clone(),
                    state: r.nfa.execution_state(),
                }
            })
            .collect::<Vec<_>>();

        let matches = vec![None; rules.len()];

        let mut ignore_chars: [bool; 256] = [false; 256];
        for s in lexicon.ignore_chars.iter() {
            let b = s.bytes().nth(0).unwrap();
            if b.is_ascii() {
                ignore_chars[b as usize] = true;
            }
        }

        let mut prefixes = Vec::with_capacity(256);
        for i in 0..256 {
            let mut matching_rules = BitSet::new();

            let c = (i as u8) as char;
            if c.is_ascii() {
                for (rule_idx, rule) in rules.iter_mut().enumerate() {
                    rule.nfa.initialize_states(&mut rule.state.current);
                    rule.nfa.step(&rule.state.current, c, &mut rule.state.next);

                    if !rule.nfa.is_dead_state(&rule.state.next) {
                        matching_rules.insert(rule_idx);
                    }
                }
            }

            prefixes.push(matching_rules);
        }

        Self {
            input,
            offset: 0,
            rules,
            matches,
            ignore_chars,
            prefixes,
        }
    }

    fn eof(&self) -> bool {
        self.offset >= self.input.len()
    }

    pub fn next(&mut self) -> Next {
        while self.offset < self.input.len() {
            let input = &self.input[self.offset..];
            let b = input.bytes().nth(0).unwrap();

            if !self.ignore_chars[b as usize] {
                break;
            }
            self.offset += 1;
        }

        if self.offset >= self.input.len() {
            return Next::End;
        }

        let input = &self.input[self.offset..];
        let b = input.bytes().nth(0).unwrap();

        let rule_indicies = &self.prefixes[b as usize];

        for (i, rule) in self.rules.iter_mut().enumerate() {
            self.matches[i] = if rule_indicies.contains(i) {
                rule.nfa.longest_match(input, &mut rule.state)
            } else {
                None
            };
        }

        let mut longest = None;
        let mut rule_idx = 0;

        for (idx, m) in self.matches.iter().enumerate() {
            if let Some(m) = m {
                if *m > longest.unwrap_or(0) {
                    longest = Some(*m);
                    rule_idx = idx;
                }
            }
        }

        if longest.is_none() {
            self.offset += 1;
            return Next::Error(Error::UnexpectedChar(&input[0..1]));
        }

        let len = longest.unwrap();

        let text = &self.input[self.offset..(self.offset + len)];
        self.offset += len;

        Next::Token(self.rules[rule_idx].id, text)
    }
}

    // pub fn next_old(&mut self) -> Next {
    //     while !self.eof() {
    //         let input = &self.input[self.offset..];

    //         let b = input.bytes().nth(0).unwrap();

    //         if b.is_ascii() {
    //             if self.ignore_chars_ascii[b as usize] {
    //                 self.offset += 1;
    //                 continue;
    //             }
    //         }

    //         let g = input.graphemes(true).nth(0).unwrap();
    //         if self.lexicon.ignore_chars.contains(g) {
    //             self.offset += g.len();
    //             continue;
    //         }

    //         let matched = self.patterns.captures_read(&mut self.locations, input);
    //         if matched.is_none() {
    //             self.offset += g.len();
    //             return Next::Error(Error::UnexpectedChar(g));
    //         }

    //         let match_len = matched.map(|m| m.end() - m.start()).unwrap();

    //         let mut rule = &self.lexicon.rules[0];
    //         for i in 1..=self.lexicon.rules.len() {
    //             if self.locations.get(i).is_some() {
    //                 rule = &self.lexicon.rules[i - 1];
    //                 break;
    //             }
    //         }

    //         self.offset += match_len;

    //         return Next::Token(rule.id(), &input[..match_len]);
    //     }

    //     Next::End
    // }
// }

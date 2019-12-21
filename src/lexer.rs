use bit_set::BitSet;

use crate::nfa::{analyze, ExecutionState, NFA};
use crate::Lexicon;

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
        let rules = lexicon
            .rules
            .iter()
            .map(|r| Rule {
                id: r.id(),
                nfa: r.nfa().clone(),
                state: r.nfa().execution_state(),
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

        let mut prefixes = vec![BitSet::new(); 256];
        for (rule_idx, rule) in rules.iter().enumerate() {
            let ranges = analyze::starting_chars(&rule.nfa);

            for (low, high) in ranges {
                for i in (low as usize)..(high as usize) {
                    if i > 255 {
                        continue;
                    }

                    prefixes[i].insert(rule_idx);
                }
            }
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

    #[inline]
    fn eof(&self) -> bool {
        self.offset >= self.input.len()
    }

    pub fn next(&mut self) -> Next {
        while !self.eof() {
            let input = &self.input[self.offset..];
            let b = input.bytes().nth(0).unwrap();

            if !self.ignore_chars[b as usize] {
                break;
            }
            self.offset += 1;
        }

        if self.eof() {
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

    pub fn reset(&mut self) {
        self.offset = 0;
    }
}

use std::char;

use bit_set::BitSet;
use hashbrown::{HashMap, HashSet};

use crate::nfa::{analyze, ExecutionState, NFA};
use crate::Lexicon;

pub struct Lexer<'input> {
    input: &'input str,
    offset: usize,
    rules: Vec<Rule>,
    matches: Vec<Option<usize>>,
    ignore_chars: HashSet<char>,
    prefixes: HashMap<char, BitSet>,
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

        let mut ignore_chars = HashSet::new();
        for s in lexicon.ignore_chars.iter() {
            let c = s.chars().nth(0).unwrap();
            ignore_chars.insert(c);
        }

        let mut prefixes = HashMap::new();
        for (rule_idx, rule) in rules.iter().enumerate() {
            let ranges = analyze::starting_chars(&rule.nfa);

            for (low, high) in ranges {
                for i in (low as u32)..(high as u32) {
                    if let Some(c) = char::from_u32(i) {
                        let rule_prefixes = prefixes.entry(c).or_insert_with(BitSet::new);
                        rule_prefixes.insert(rule_idx);
                    }
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

    pub fn next(&mut self) -> Next {
        let mut c = self.input[self.offset..].chars().nth(0);
        while let Some(ch) = c {
            if !self.ignore_chars.contains(&ch) {
                break;
            }

            self.offset += ch.len_utf8();
            c = self.input[self.offset..].chars().nth(0);
        }

        if c.is_none() {
            return Next::End;
        }

        let c = c.unwrap();
        let input = &self.input[self.offset..];

        let rule_indicies = match self.prefixes.get(&c) {
            Some(indicies) => indicies,
            None => {
                self.offset += c.len_utf8();
                return Next::Error(Error::UnexpectedChar(&input[0..c.len_utf8()]));
            }
        };

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
            self.offset += c.len_utf8();
            return Next::Error(Error::UnexpectedChar(&input[0..c.len_utf8()]));
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

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

    #[inline]
    fn eof(&self) -> bool {
        self.offset >= self.input.len()
    }

    pub fn next(&mut self) -> Next {
        while !self.eof() {
            let input = &self.input[self.offset..];
            let c = input.chars().nth(0).unwrap();

            if !self.ignore_chars.contains(&c) {
                break;
            }
            self.offset += c.len_utf8();
        }

        if self.eof() {
            return Next::End;
        }

        let input = &self.input[self.offset..];
        let c = input.chars().nth(0).unwrap();

        let rule_indicies = self.prefixes.get(&c);
        if rule_indicies.is_none() {
            self.offset += c.len_utf8();
            return Next::Error(Error::UnexpectedChar(&input[0..c.len_utf8()]));
        }

        let rule_indicies = rule_indicies.unwrap();

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

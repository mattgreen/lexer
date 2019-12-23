use std::char;

use bit_set::BitSet;
use hashbrown::{HashMap, HashSet};

use crate::nfa::{analyze, ExecutionState, NFA};
use crate::Lexicon;

pub struct Lexer<'input> {
    input: &'input str,
    offset: usize,
    pos: Position,
    rules: Vec<Rule>,
    matches: Vec<Option<(usize, Position)>>,
    ignore_chars: HashSet<char>,
    prefixes: HashMap<char, BitSet>,
}

#[derive(Debug, PartialEq)]
pub enum Next<'input> {
    Token(usize, &'input str, Position),
    End,
    Error(Error<'input>, Position),
}

#[derive(Debug, PartialEq)]
pub enum Error<'input> {
    UnexpectedChar(&'input str),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    line: u32,
    col: u32,
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
                for i in (low as u32)..((high as u32) + 1) {
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
            pos: Position { line: 1, col: 1 },
            rules,
            matches,
            ignore_chars,
            prefixes,
        }
    }

    fn advance(&mut self, ch: char) {
        self.offset += ch.len_utf8();

        if ch == '\n' {
            self.pos.line += 1;
            self.pos.col = 1;
        } else {
            self.pos.col += 1;
        }
    }

    fn longest_match(nfa: &NFA, input: &str, state: &mut ExecutionState, start: Position) -> Option<(usize, Position)> {
        let mut end = start;

        nfa.initialize_states(&mut state.current);

        let mut match_len = None;
        for (len, c) in input.chars().enumerate() {
            nfa.step(&state.current, c, &mut state.next);
            if c == '\n' {
                end.line += 1;
                end.col = 1;
            } else {
                end.col += 1;
            }

            if nfa.has_match_state(&state.next) {
                match_len = Some((len + 1, end));
            } else if nfa.is_dead_state(&state.next) {
                break;
            }

            std::mem::swap(&mut state.current, &mut state.next);
        }

        match_len
    }

    pub fn next(&mut self) -> Next {
        let c = loop {
            match self.input[self.offset..].chars().nth(0) {
                Some(ch) => {
                    if !self.ignore_chars.contains(&ch) {
                        break ch;
                    }

                    self.advance(ch);
                },
                None => return Next::End,
            }
        };

        let input = &self.input[self.offset..];
        let pos = self.pos;

        let rule_indicies = match self.prefixes.get(&c) {
            Some(indicies) => indicies,
            None => {
                self.advance(c);

                return Next::Error(Error::UnexpectedChar(&input[0..c.len_utf8()]), pos);
            }
        };

        for (i, rule) in self.rules.iter_mut().enumerate() {
            self.matches[i] = if rule_indicies.contains(i) {
                Self::longest_match(&rule.nfa, input, &mut rule.state, pos)
                // rule.nfa.longest_match(input, &mut rule.state)
            } else {
                None
            };
        }

        let mut longest = None;
        let mut rule_idx = 0;
        let mut end_pos = pos;

        for (idx, m) in self.matches.iter().enumerate() {
            if let Some((m, end)) = m {
                if *m > longest.unwrap_or(0) {
                    longest = Some(*m);
                    rule_idx = idx;
                    end_pos = *end;
                }
            }
        }

        if longest.is_none() {
            self.advance(c);

            return Next::Error(Error::UnexpectedChar(&input[0..c.len_utf8()]), self.pos);
        }

        let len = longest.unwrap();

        let text = &self.input[self.offset..(self.offset + len)];
        self.offset += len;
        self.pos = end_pos;

        Next::Token(self.rules[rule_idx].id, text, pos)
    }

    pub fn reset(&mut self) {
        self.offset = 0;
        self.pos = Position::new(1, 1);
    }
}

impl Position {
    pub fn new(line: u32, col: u32) -> Position {
        Position { line, col }
    }
}
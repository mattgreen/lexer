use std::char;
use std::iter::FromIterator;

use fixedbitset::FixedBitSet;
use hashbrown::{HashMap, HashSet};
use regex::Regex;

use crate::lexicon::{Lexicon, RuleID, RuleKind};

pub struct Lexer<'input> {
    input: &'input str,
    offset: usize,
    pos: Position,
    rules: Vec<Rule>,
    matches: Vec<(usize, usize)>,
    ignore_chars: HashSet<char>,
    prefixes: HashMap<char, FixedBitSet>,
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
    kind: RuleKind,
    regex: Regex,
}

impl<'input> Lexer<'input> {
    pub fn new(lexicon: &Lexicon, input: &'input str) -> Self {
        let rules = lexicon
            .rules
            .iter()
            .map(|r| {
                let pattern = {
                    if r.kind == RuleKind::Literal {
                        r.pattern
                            .to_owned()
                            .replace("\\", "\\\\")
                            .replace("?", "\\?")
                            .replace("(", "\\(")
                            .replace(")", "\\)")
                            .replace("{", "\\{")
                            .replace("}", "\\}")
                            .replace("[", "\\[")
                            .replace("]", "\\]")
                    } else {
                        r.pattern.to_owned()
                    }
                };

                let anchored = format!("\\A(?:{})", pattern);
                let regex = Regex::new(&anchored).unwrap();

                Rule {
                    id: r.id,
                    kind: r.kind,
                    regex,
                }
            })
            .collect::<Vec<_>>();

        let ignore_chars = HashSet::from_iter(lexicon.ignore_chars.iter().copied());

        let mut prefixes = HashMap::new();
        for (rule_idx, rule) in lexicon.rules.iter().enumerate() {
            for c in rule.starting_chars.iter() {
                prefixes
                    .entry(*c)
                    .or_insert_with(|| FixedBitSet::with_capacity(rules.len()))
                    .insert(rule_idx);
            }
        }

        Self {
            input,
            offset: 0,
            pos: Position { line: 1, col: 1 },
            rules,
            matches: Vec::with_capacity(lexicon.rules.len()),
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

    fn best_match(&self, matches: &[(usize, usize)]) -> Option<(RuleID, usize)> {
        if matches.is_empty() {
            return None;
        }

        let mut kind = RuleKind::Pattern;
        let mut match_len = 0;
        let mut rule_id = 0;

        for (i, len) in matches.iter() {
            let rule = &self.rules[*i];
            if *len > match_len || (*len == match_len && rule.kind > kind) {
                kind = rule.kind;
                rule_id = rule.id;
                match_len = *len;
            }
        }

        Some((rule_id, match_len))
    }

    pub fn next(&mut self) -> Next {
        let c = loop {
            match self.input[self.offset..].chars().nth(0) {
                Some(ch) => {
                    if !self.ignore_chars.contains(&ch) {
                        break ch;
                    }

                    self.advance(ch);
                }
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

        self.matches.clear();

        for i in rule_indicies.ones() {
            let rule = &self.rules[i];

            if let Some(m) = rule.regex.find_at(input, 0) {
                self.matches.push((i, m.end()));
            }
        }

        let best = self.best_match(&self.matches);
        if best.is_none() {
            self.advance(c);
            return Next::Error(Error::UnexpectedChar(&input[0..c.len_utf8()]), pos);
        }

        let (rule_id, len) = best.unwrap();
        let text = &self.input[self.offset..(self.offset + len)];

        for c in input[..len].chars() {
            self.advance(c);
        }

        Next::Token(rule_id, text, pos)
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

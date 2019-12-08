use std::collections::HashSet;

use regex::Regex;

pub struct Lexicon<'input, T> {
    pub(crate) ignore_chars: HashSet<char>,
    pub(crate) rules: Vec<Rule<'input, T>>,
}

#[derive(Default)]
pub struct LexiconBuilder<'input, T> {
    ignore_chars: HashSet<char>,
    rules: Vec<Rule<'input, T>>,
}

pub struct Rule<'input, T> {
    pub(crate) regex: Regex,
    pub(crate) handler: Option<Handler<'input, T>>,
}

pub type Handler<'input, T> = fn(&'input str) -> T;

#[derive(Debug)]
pub enum Error {
    Regex(regex::Error),
}

impl<'input, T> LexiconBuilder<'input, T> {
    pub fn new() -> Self {
        Self { ignore_chars: HashSet::new(), rules: vec![] }
    }

    pub fn build(self) -> Lexicon<'input, T> {
        Lexicon { ignore_chars: self.ignore_chars, rules: self.rules }
    }

    pub fn ignore_chars(mut self, chars: &str) -> Self {
        self.ignore_chars.extend(chars.chars());

        self
    }

    pub fn ignore_regex(mut self, pattern: &str) -> Result<Self, Error> {
        let regex = Regex::new(pattern).map_err(Error::Regex)?;

        self.rules.push(Rule { regex, handler: None });

        Ok(self)
    }

    pub fn token(mut self, pattern: &str, handler: Handler<'input, T>) -> Result<Self, Error> {
        let regex = Regex::new(pattern).map_err(Error::Regex)?;

        self.rules.push(Rule { regex, handler: Some(handler) });

        Ok(self)
    }
}
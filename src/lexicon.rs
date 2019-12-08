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

pub enum Rule<'input, T> {
    Ignore(Regex),
    Token(Regex, Handler<'input, T>),
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
        let anchored = Self::anchor(pattern);
        let regex = Regex::new(&anchored).map_err(Error::Regex)?;

        self.rules.push(Rule::Ignore(regex));

        Ok(self)
    }

    pub fn token(mut self, pattern: &str, handler: Handler<'input, T>) -> Result<Self, Error> {
        let anchored = Self::anchor(pattern);
        let regex = Regex::new(&anchored).map_err(Error::Regex)?;

        self.rules.push(Rule::Token(regex, handler));

        Ok(self)
    }

    fn anchor(pattern: &str) -> String {
        let mut anchored = pattern.to_owned();
        if !anchored.starts_with('^') && !anchored.starts_with("\\A") {
            anchored.insert(0, '^');
        }

        anchored
    }
}

impl<'input, T> Rule<'input, T> {
    pub(crate) fn match_len(&self, input: &str) -> Option<usize> {
        let regex = match self {
            Self::Ignore(r) => r,
            Self::Token(r, _) => r,
        };

        regex.find(input).map(|m| m.end() - m.start())
    }
}
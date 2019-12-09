use std::collections::HashSet;

use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

pub struct Lexicon {
    pub(crate) ignore_chars: HashSet<String>,
    pub(crate) rules: Vec<Rule>,
}

#[derive(Default)]
pub struct LexiconBuilder {
    ignore_chars: HashSet<String>,
    rules: Vec<Rule>,
}

pub struct Rule {
    pub(crate) id: usize,
    pub(crate) regex: Regex,
}

#[derive(Debug)]
pub enum Error {
    Regex(regex::Error),
}

impl LexiconBuilder {
    pub fn new() -> Self {
        Self { ignore_chars: HashSet::new(), rules: vec![] }
    }

    pub fn build(self) -> Lexicon {
        Lexicon { ignore_chars: self.ignore_chars, rules: self.rules }
    }

    pub fn ignore_chars(mut self, chars: &str) -> Self {
        let new_chars = chars.graphemes(true).map(String::from).collect::<Vec<String>>();
        self.ignore_chars.extend(new_chars);

        self
    }

    pub fn rule(mut self, id: usize, pattern: &str) -> Result<Self, Error> {
        let regex = Regex::new(pattern).map_err(Error::Regex)?;

        self.rules.push(Rule { id, regex });

        Ok(self)
    }
}
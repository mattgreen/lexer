use std::collections::HashSet;

pub struct Lexicon {
    pub(crate) ignore_chars: HashSet<char>,
    pub(crate) rules: Vec<Rule<String>>,
}

#[derive(Default)]
pub struct LexiconBuilder {
    ignore_chars: HashSet<char>,
    rules: Vec<Rule<String>>,
}

pub(crate) struct Rule<T> {
    pub(crate) id: usize,
    pub(crate) kind: RuleKind<T>,
}

pub(crate) enum RuleKind<T> {
    Pattern(T),
    Literal(T),
}

#[derive(Debug)]
pub enum Error {}

impl LexiconBuilder {
    pub fn new() -> Self {
        Self {
            ignore_chars: HashSet::new(),
            rules: vec![],
        }
    }

    pub fn build(self) -> Result<Lexicon, Error> {
        Ok(Lexicon {
            ignore_chars: self.ignore_chars,
            rules: self.rules,
        })
    }

    pub fn ignore_chars(mut self, chars: &str) -> Self {
        for ch in chars.chars() {
            self.ignore_chars.insert(ch);
        }

        self
    }

    pub fn literal(mut self, id: usize, literal: &str) -> Self {
        self.rules.push(Rule { id, kind: RuleKind::Literal(literal.into()) });

        self
    }

    pub fn pattern(mut self, id: usize, pattern: &str) -> Self {
        self.rules.push(Rule { id, kind: RuleKind::Pattern(pattern.into()) });

        self
    }
}
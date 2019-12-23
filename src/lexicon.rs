use std::collections::HashSet;

use crate::nfa;

pub struct Lexicon {
    pub(crate) ignore_chars: HashSet<char>,
    pub(crate) rules: Vec<Rule>,
}

#[derive(Default)]
pub struct LexiconBuilder {
    ignore_chars: HashSet<char>,
    rules: Vec<Rule>,
}

pub(crate) struct Rule {
    id: usize,
    nfa: nfa::NFA,
}

pub type Error = nfa::CompileError;

impl LexiconBuilder {
    pub fn new() -> Self {
        Self {
            ignore_chars: HashSet::new(),
            rules: vec![],
        }
    }

    pub fn build(self) -> Lexicon {
        Lexicon {
            ignore_chars: self.ignore_chars,
            rules: self.rules,
        }
    }

    pub fn ignore_chars(mut self, chars: &str) -> Self {
        for ch in chars.chars() {
            self.ignore_chars.insert(ch);
        }

        self
    }

    pub fn rule(mut self, id: usize, pattern: &str) -> Result<Self, Error> {
        let nfa = nfa::compile(pattern)?;

        self.rules.push(Rule { id, nfa });

        Ok(self)
    }
}

impl Rule {
    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn nfa(&self) -> &nfa::NFA {
        &self.nfa
    }
}

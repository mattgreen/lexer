use hashbrown::HashSet;

use crate::nfa::{analyze, CompileError, NFA};

pub struct Lexicon {
    pub(crate) ignore_chars: HashSet<char>,
    pub(crate) rules: Vec<Rule>,
}

#[derive(Default)]
pub struct LexiconBuilder {
    ignore_chars: HashSet<char>,
    rules: Vec<(RuleID, RuleKind, String)>,
}

pub(crate) struct Rule {
    pub(crate) id: RuleID,
    pub(crate) precedence: u8,
    pub(crate) pattern: String,
    pub(crate) starting_chars: HashSet<char>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(crate) enum RuleKind {
    Pattern,
    Literal,
}

pub type RuleID = usize;

pub type Error = CompileError;

impl LexiconBuilder {
    pub fn new() -> Self {
        Self {
            ignore_chars: HashSet::new(),
            rules: vec![],
        }
    }

    pub fn build(self) -> Result<Lexicon, Error> {
        let mut rules = vec![];
        for (id, kind, pattern) in self.rules {
            let nfa = NFA::from_regex(&pattern)?;
            let starting_chars = analyze::starting_chars(&nfa);
            let precedence = if kind == RuleKind::Literal { 1 } else { 0 };

            rules.push(Rule {
                id,
                precedence,
                pattern,
                starting_chars,
            });
        }

        Ok(Lexicon {
            ignore_chars: self.ignore_chars,
            rules,
        })
    }

    pub fn ignore_chars(mut self, chars: &str) -> Self {
        for ch in chars.chars() {
            self.ignore_chars.insert(ch);
        }

        self
    }

    pub fn literal(mut self, id: RuleID, literal: &str) -> Self {
        let pattern = literal
            .to_owned()
            .replace("\\", "\\\\")
            .replace("?", "\\?")
            .replace("(", "\\(")
            .replace(")", "\\)")
            .replace("{", "\\{")
            .replace("}", "\\}")
            .replace("[", "\\[")
            .replace("]", "\\]");

        self.rules.push((id, RuleKind::Literal, pattern));

        self
    }

    pub fn pattern(mut self, id: RuleID, pattern: &str) -> Self {
        self.rules.push((id, RuleKind::Pattern, pattern.into()));

        self
    }
}

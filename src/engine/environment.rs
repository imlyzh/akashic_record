use std::sync::RwLock;

use crate::structs::{fact::FactRecord, rule::RuleRecord};

#[derive(Debug, Default)]
pub struct Database {
    pub facts: RwLock<FactRecord>,
    pub rules: RwLock<RuleRecord>,
}

// pub type Env = (Handle<Database>, Handle<Scope>);

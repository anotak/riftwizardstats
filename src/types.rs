use std::collections::HashMap;
use std::collections::HashSet;

use crate::hashmap::HashMapExtensions;

#[derive(Default,Debug)]
pub struct Realm
{
    pub realm_number : i64,
    pub outcome : Outcome,
    pub turns_taken_realm : i64,
    pub turns_taken_run : i64,
    pub spell_casts : Option<HashMap<String, usize>>,
    pub damage_to_enemies : Option<HashMap<String, usize>>,
    pub damage_to_wizard : Option<HashMap<String, usize>>,
    pub items_used : Option<HashMap<String, usize>>,
    pub purchases : Option<HashSet<String>>,
}

#[derive(Default,Debug)]
pub struct Run
{
    pub realms : i64,
    pub outcome : Outcome,
    pub turns_taken_run : i64,
    pub spell_casts : Option<HashMap<String, usize>>,
    pub damage_to_enemies : Option<HashMap<String, usize>>,
    pub damage_to_wizard : Option<HashMap<String, usize>>,
    pub items_used : Option<HashMap<String, usize>>,
    pub purchases : Option<HashMap<String, usize>>,
}

#[derive(Default,Debug)]
pub struct MergedRuns
{
    pub realms : i64,
    pub num_victory : usize,
    pub num_defeat : usize,
    pub num_abandoned : usize,
    pub num_unknown : usize,
    pub turns_taken : i64,
    pub spell_casts : Option<HashMap<String, usize>>,
    pub damage_to_enemies : Option<HashMap<String, usize>>,
    pub damage_to_wizard : Option<HashMap<String, usize>>,
    pub items_used : Option<HashMap<String, usize>>,
    pub purchases : Option<HashMap<Purchase, usize>>,
}

#[derive(Default,Debug)]
pub struct Purchase
{
    pub name : String,
    pub outcome : Outcome,
    pub realm : i64,
}

impl MergedRuns {
    pub fn merge_run(self, new_run : Run) -> MergedRuns
    {
        let mut output = self;
        
        output.realms += new_run.realms;
        
        match new_run.outcome {
            Outcome::Victory => output.num_victory += 1,
            Outcome::Defeat => output.num_defeat += 1,
            Outcome::Abandoned => output.num_abandoned += 1,
            Outcome::Unknown => output.num_unknown += 1,
        }
        
        output.turns_taken += new_run.turns_taken_run;
        
        output.spell_casts = output.spell_casts.merge_add(new_run.spell_casts);
        output.damage_to_enemies = output.damage_to_enemies.merge_add(new_run.damage_to_enemies);
        output.damage_to_wizard = output.damage_to_wizard.merge_add(new_run.damage_to_wizard);
        output.items_used = output.items_used.merge_add(new_run.items_used);
        
        output
    }
}

#[derive(Debug, PartialEq)]
pub enum Outcome
{
    Victory,
    Defeat,
    Unknown,
    Abandoned
}

impl Default for Outcome {
    fn default() -> Self { Outcome::Unknown }
}
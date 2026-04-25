use std::{collections::{HashMap, btree_map::Range}, sync::LazyLock};

use clap::builder::Str;

static DERIVE_OPTIONS: LazyLock<HashMap<&'static str, fn(&str, &str) -> DeriveScope>> = LazyLock::new(||{
    let mut hm: HashMap<&'static str, fn(&str, &str) -> DeriveScope> = HashMap::new();
    hm.insert("def", def);
    hm.insert("var", arrow_var);
    hm.insert("regex", regex);
    hm
});

pub fn apply_options(var: &str, matched: &str, features: &Vec<String>) -> Vec<DeriveScope>{
    let mut res:Vec<DeriveScope>= Vec::new();
    features.iter().for_each(|feature|{
        let opt = DERIVE_OPTIONS.get(feature.as_str());
        if opt.is_none() {
            panic!("No such derive option named {}",feature)
        }
        res.push(opt.as_ref().unwrap()(var,matched));
    });
    res
}

pub enum DeriveScope {
    Before(String),
    After(String),
    Replace(String),
    Arround(String,String),
    None,
}

// var and caught pattern
pub fn def(var: &str, _: &str) -> DeriveScope {
    return DeriveScope::Arround(format!("//- def {}: \n", var), format!("\n //- endef: \n"));
}
pub fn arrow_var(var: &str, _:&str) -> DeriveScope{
    return DeriveScope::Before(format!("/*- $#{} -> -*/ ", var));
}
pub fn regex(_: &str, matched:&str) -> DeriveScope{
    DeriveScope::None
}


use std::{collections::HashMap, ops::Range, sync::LazyLock};

use crate::structs::VarOption;

type ArgType = String;
type FnReturn =  Option<Vec<(Range<usize>, DeriveScope)>>;
type FnType = fn(&str, &str,&Range<usize>, &Vec<String>) -> FnReturn;
type MapType = HashMap<&'static str, FnType>;

static DERIVE_OPTIONS: LazyLock<MapType> = LazyLock::new(||{
    let mut hm: MapType = HashMap::new();
    hm.insert("def", def);
    hm.insert("var", arrow_var);
    hm.insert("regex", regex);
    hm.insert("placea", placea);
    hm.insert("placeb", placeb);
    hm
});

pub fn apply_options(var: &str, matched: &str, range: &Range<usize>,features: &Vec<VarOption>) -> FnReturn{
    let mut res: FnReturn = None;
    features.iter().for_each(|feature|{
        let opt = DERIVE_OPTIONS.get(feature.option.as_str());
        if opt.is_none() {
            panic!("No such derive option named {}",feature.option)
        }
        match opt.as_ref().unwrap()(var,matched,range, &feature.args) {
            Some(mut replaces) => {
                if res.is_none() {
                    res = Some(Vec::new());
                }
                res.as_mut().unwrap().append(&mut replaces);
            },
            None => (),
        }
    });
    res
}

pub enum DeriveScope {
    Before(String),
    After(String),
    Replace(String),
    None,
}

// var and caught pattern
pub fn def(var: &str, _: &str, range: &Range<usize>, args: &Vec<String>) -> FnReturn {
    let start = range.start..range.start;
    let end = range.end..range.end;
    return Some(vec![
        (start,DeriveScope::Before(format!("//- def {}: \n", var))), 
        (end, DeriveScope::After(format!("\n //- endef: \n")))
    ]);
}
pub fn arrow_var(var: &str, _:&str, range: &Range<usize>, args: &Vec<ArgType>) -> FnReturn{
    return Some(vec![(range.clone(),DeriveScope::Before(format!("/*- $#{} -> -*/ ", var)))]);
}
pub fn regex(_: &str, _:&str, _ : &Range<usize>, args: &Vec<ArgType>) -> FnReturn{
    None
}
pub fn placea(var: &str, _:&str, range: &Range<usize>, args: &Vec<ArgType>) -> FnReturn{
    return Some(vec![(range.clone(), DeriveScope::After(format!("\n//- place {var}:\n")))]);
}
pub fn placeb(var: &str, _:&str, range: &Range<usize>, args: &Vec<ArgType>) -> FnReturn{
    return Some(vec![(range.clone(), DeriveScope::Before(format!("\n//- place {var}:\n")))]);
}

use std::{collections::HashMap, sync::{LazyLock}};

static DERIVE_OPTIONS: LazyLock<HashMap<&'static str, fn(&str, &str) -> String>> = LazyLock::new(||{
    let mut hm: HashMap<&'static str, fn(&str, &str) -> String> = HashMap::new();
    hm.insert("def", def);
    hm
});

pub fn apply_options(var: &str, matched: &str, features: &Vec<String>) -> String{
    let mut res:String = matched.to_string();
    features.iter().for_each(|feature|{
        let opt = DERIVE_OPTIONS.get(feature.as_str());
        if opt.is_none() {
            panic!("No such derive option named {}",feature)
        }
        res = opt.as_ref().unwrap()(var,&res);
    });
    res
}

// var and caught pattern
pub fn def(var: &str, matched: &str) -> String {
    let before = format!("//- def {}: {} //- endef:", var, matched);
    before
}
pub fn arrow_var(var: &str, matched:&str) -> String{
    let before = format!("/*- $#{} -> -*/ {}", var,matched);
    before
}
pub fn regex(_: &str, matched:&str) -> String{
    matched.to_string()
}


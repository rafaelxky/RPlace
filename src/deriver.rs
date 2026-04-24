use std::{collections::HashMap, fs, ops::Range};

use crate::{parser::VarOptions, writer::Derive};
use regex::Regex;

pub struct Deriver {}
impl Deriver {
    // todo: resolve variable derive
    pub fn derive(derive: &Derive) -> String{
        let text = fs::read_to_string(derive.path.clone());
        if text.is_err() {
            panic!("Error: no such file {} for derive", derive.path)
        }
        let mut text = text.unwrap();
        let mut all_changes: Vec<(Range<usize>, String)> = Vec::new();

        derive.vals.iter().for_each(|(var, pattern)| {
            let opts = &pattern.options;
            let features =  if opts.is_some() {
                Some(Deriver::get_features(&opts.as_ref().unwrap()))
            } else {
                None
            };

            if features.is_some() && features.as_ref().unwrap().contains_key("regex"){
                let re = Regex::new(&pattern.value).unwrap();
                for mat in re.find_iter(&text) {
                    if !all_changes
                        .iter()
                        .any(|(r, _)| r.contains(&mat.start()) || r.contains(&mat.end()))
                    {
                        let replacement = format!("/*- $#{} -> -*/ {}", var, &text[mat.range()]);
                        all_changes.push((mat.range(), replacement));
                    }
                }
            } else {
                let mut start = 0;
                while let Some(pos) = text[start..].find(&pattern.value) {
                    let abs = start + pos;
                    let range = abs..abs + pattern.value.len();
                    if !all_changes
                        .iter()
                        .any(|(r, _)| r.contains(&abs) || r.contains(&range.end))
                    {
                        let replacement = format!("/*- $#{} -> -*/ {}", var, &pattern.value);
                        all_changes.push((range, replacement));
                    }
                    start = abs + pattern.value.len();
                }
            }
        });

        all_changes.sort_by(|a, b| b.0.start.cmp(&a.0.start));
        for (range, replacement) in all_changes {
            text.replace_range(range, &replacement);
        }
        return text;
    }
    fn get_features(opts: &Vec<VarOptions>) -> HashMap<String, usize> {
        let mut map: HashMap<String, usize> = HashMap::new();
        opts.iter().enumerate().for_each(|(i, opt)| {
            match opt {
                VarOptions::Regex => map.insert("regex".to_string(), i),
            };
        });
        map
    }
}

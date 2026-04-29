use std::{
    collections::HashMap,
    fs,
    ops::{Range},
};

use crate::{
    derive_options::options_map::{DeriveScope, apply_options, arrow_var},
    writer::Derive,
};
use regex::Regex;

pub struct Deriver {}
impl Deriver {
    // todo: resolve variable derive
    pub fn derive(derive: &Derive) -> String {
        let text = fs::read_to_string(derive.path.clone());
        if text.is_err() {
            panic!("Error: no such file {} for derive", derive.path)
        }
        let text = text.unwrap();
        let mut all_changes: Vec<(Range<usize>, DeriveScope)> = Vec::new();

        derive.vals.iter().for_each(|(var, pattern)| {
            let opts = pattern.options.as_ref();

            let features = opts.map(|o| Deriver::get_features(o));
            let features_vec = opts;

            let is_regex = features
                .as_ref()
                .map(|f| f.contains_key("regex"))
                .unwrap_or(false);

            let default_place = opts
                .map(|o| !o.iter().any(|opt| opt.as_str() != "regex"))
                .unwrap_or(true);

            if is_regex {
                let re = Regex::new(&pattern.value).unwrap();

                for mat in re.find_iter(&text) {
                    if mat.start() == mat.end() {
                        continue;
                    }

                    let matched = &text[mat.range()];

                    let mut replacement = if default_place {
                        arrow_var(var, matched, &mat.range())
                    } else {
                        apply_options(var, matched, &mat.range(), features_vec.unwrap())
                    };

                    if replacement.is_some() {
                        all_changes.append(replacement.as_mut().unwrap());
                    }
                }
            } else {
                let mut start = 0;

                while let Some(pos) = text[start..].find(&pattern.value) {
                    let abs = start + pos;
                    let range = abs..abs + pattern.value.len();

                    let matched = &text[range.clone()];

                    let mut replacement = if default_place {
                        arrow_var(var, matched, &range)
                    } else {
                        apply_options(var, matched, &range, features_vec.unwrap())
                    };

                    if replacement.is_some() {
                        all_changes.append(replacement.as_mut().unwrap());
                    }

                    start = abs + pattern.value.len();
                }
            }
        });

        all_changes.sort_by(|a, b| b.0.start.cmp(&a.0.start));
        let mut text = text;
        for (range, replacement) in all_changes {
            match replacement {
                DeriveScope::Before(val) => {
                    text.insert_str(range.start, &val);
                }
                DeriveScope::After(val) => {
                    text.insert_str(range.end, &val);
                }
                DeriveScope::Replace(val) => {
                    text.replace_range(range.clone(), &val);
                }
                DeriveScope::None => {}
            }
        }
        return text;
    }
    fn get_features(opts: &Vec<String>) -> HashMap<String, usize> {
        let mut map: HashMap<String, usize> = HashMap::new();
        opts.iter().enumerate().for_each(|(i, opt)| {
            map.insert(opt.clone(), i);
        });
        map
    }
}

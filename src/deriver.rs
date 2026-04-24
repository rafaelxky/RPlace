use std::{
    collections::HashMap,
    fs,
    ops::{Range, RangeFrom},
};

use crate::{
    derive_options::options_map::{apply_options, arrow_var},
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
        let mut text = text.unwrap();
        let mut all_changes: Vec<(Range<usize>, String)> = Vec::new();

        derive.vals.iter().for_each(|(var, pattern)| {
            let mut default_place = true;
            let opts = &pattern.options;
            let features = if opts.is_some() {
                Some(Deriver::get_features(&opts.as_ref().unwrap()))
            } else {
                None
            };
            let features_vec = if opts.is_some() {
                default_place = !opts.as_ref().unwrap().iter().any(|opt|opt.as_str() != "regex");
                Some(opts.as_ref().unwrap())
            } else {
                None
            };

            // regex
            if features.is_some() && features.as_ref().unwrap().contains_key("regex") {
                let re = Regex::new(&pattern.value).unwrap();
                for mat in re.find_iter(&text) {
                    if !all_changes
                        .iter()
                        .any(|(r, _)| r.contains(&mat.start()) || r.contains(&mat.end()))
                    {
                        if default_place {
                            all_changes
                                .push((mat.range(), arrow_var(var, &text[mat.range()])));
                        } else {
                            all_changes.push((mat.range(), apply_options(var, &text[mat.range()], features_vec.unwrap())));
                        }
                    }
                }
                // no regex
            } else {
                let mut start = 0;
                while let Some(pos) = text[start..].find(&pattern.value) {
                    let abs = start + pos;
                    let range = abs..abs + pattern.value.len();
                    if !all_changes
                        .iter()
                        .any(|(r, _)| r.contains(&abs) || r.contains(&range.end))
                    {
                        if default_place {
                            all_changes.push((
                                range.clone(),
                                arrow_var(var, &text[range.clone()]),
                            ));
                        } else {
                            features_vec.as_ref().unwrap().iter().for_each(|feature| {
                                all_changes
                                    .push((range.clone(), apply_options(var, &text[range.clone()], features_vec.unwrap())));
                            });
                        }
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
    fn get_features(opts: &Vec<String>) -> HashMap<String, usize> {
        let mut map: HashMap<String, usize> = HashMap::new();
        opts.iter().enumerate().for_each(|(i, opt)| {
            map.insert(opt.clone(), i);
        });
        map
    }
}

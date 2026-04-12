pub fn handle_error<S: Into<String>>(msg: S, line: usize, file: S) -> ! {
    let mut msg = msg.into();
    let mut chars = msg.chars();
    msg = match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    };
    panic!("{} at line {} in file {}", msg, line + 1, file.into());
}
pub fn handle_expected_error<S: Into<String>>(expected: S, found: S, after: S, line: usize, file: S) -> ! {
    panic!(
        "Expected {} found {} {} at line {} in file {}",
        expected.into(),
        found.into(),
        after.into(),
        line,
        file.into()
    );
}

pub fn strip_until_match(remove_until: &str, input: &str) -> String {
    let mut to_strip = String::new();
    input.clone_into(&mut to_strip);
    to_strip = to_strip.chars().rev().collect();

    let mut stripped = String::new();
    while let Some(c) = to_strip.pop() {
        if stripped.to_lowercase() == remove_until.to_lowercase() {
            break;
        }
        stripped.push(c);
    }
    to_strip.chars().rev().collect()
}

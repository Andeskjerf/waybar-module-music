pub fn print_vec(input: &Vec<String>) {
    for elem in input {
        println!("{elem}");
    }
}

pub fn strip_until_backwards(remove_until: char, input: &str) -> String {
    let mut to_strip = String::new();
    input.clone_into(&mut to_strip);
    let mut stripped = String::new();
    while let Some(c) = to_strip.pop() {
        if c == remove_until {
            break;
        }
        stripped.push(c);
    }
    stripped.chars().rev().collect()
}

pub fn strip_until_match(remove_until: String, input: &str) -> String {
    let mut to_strip = String::new();
    input.clone_into(&mut to_strip);
    to_strip = to_strip.chars().rev().collect();

    let mut stripped = String::new();
    while let Some(c) = to_strip.pop() {
        if stripped == remove_until {
            break;
        }
        stripped.push(c);
    }
    to_strip.chars().rev().collect()
}

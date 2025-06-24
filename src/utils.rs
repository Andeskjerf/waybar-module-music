pub fn strip_until_match<'a>(remove_until: &'a str, input: &'a str) -> &'a str {
    let needle = remove_until.to_lowercase();
    let haystack = input.to_lowercase();

    if let Some(pos) = haystack.find(&needle) {
        &input[pos + remove_until.len()..]
    } else {
        input
    }
}

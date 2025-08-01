pub fn parse_first_number(s: &str) -> Option<u64> {
    let mut num = String::new();
    let mut found = false;

    for c in s.chars() {
        if c.is_ascii_digit() {
            num.push(c);
            found = true;
        } else if found {
            break;
        }
    }

    match found {
        true => num.parse().ok(),
        false => None,
    }
}

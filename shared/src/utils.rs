use regex::Regex;

static BLOCK_LIST: &[&str] = &["www", "uniq"];
pub fn validate_subdomain(subdomain: &str) -> Result<(), String> {
    let regex = Regex::new(r"^[a-z\d](?:[a-z\d]|-[a-z\d]){0,38}$").unwrap();
    if subdomain.len() > 38 || subdomain.len() < 3 {
        return Err(String::from("subdomain length must be between 3 and 42"));
    }
    if BLOCK_LIST.contains(&subdomain) {
        return Err(String::from("subdomain is in deny list"));
    }
    if !regex.is_match(subdomain) {
        return Err(String::from("subdomain must be lowercase & alphanumeric"));
    }
    Ok(())
}

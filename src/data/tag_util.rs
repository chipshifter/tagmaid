use anyhow::{bail, Context, Result};
use hex;
use regex::Regex;

/// Returns true if the tag string is valid and can be used in the database.
/// Returns false otherwise.
/// Conditions: Alphanumeric and paranthesis. Underscores, hypthens and aposthrophes
/// allowed but not at the beginning or the end of the string.
pub fn is_tag_name_valid(tag_name: &str) -> bool {
    let re = Regex::new(r"^[()a-zA-Z0-9]([()a-zA-Z0-9-_']*[()a-zA-Z0-9])?$").unwrap();
    re.is_match(tag_name)
}

/// Returns Ok() is tag name is valid (according to `is_tag_name_valid()`)
/// Returns Err() otherwise
pub fn validate_tag_name(tag_name: &str) -> Result<()> {
    // Errors out if tag name isn't valid
    match is_tag_name_valid(&tag_name) {
        true => Ok(()),
        false => bail!("Tag name '{}' isn't valid", &tag_name),
    }
}

/// Returns the first 16 characters of the file hash in a hexadecimal string.
/// Used for file names in the database.
pub fn trimmed_hash_hex(hash: &Vec<u8>) -> Result<String> {
    if &hash.len() >= &16 {
        let mut truncated_hash = hash.clone();
        truncated_hash.truncate(8);
        Ok(bytes_to_hex(&truncated_hash))
    } else {
        bail!("Hash is too short (invalid hash?)");
    }
}

pub fn bytes_to_hex(hash: &Vec<u8>) -> String {
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_correct_tags() {
        assert!(is_tag_name_valid("bawk"));
        assert!(is_tag_name_valid("b-bawk"));
        assert!(is_tag_name_valid("b-b-bbAWWKK"));
        assert!(is_tag_name_valid("BAWWKBAAAWKK"));
        assert!(is_tag_name_valid("B_B_BAWWKKK"));
        assert!(is_tag_name_valid("BAWK_BWAK_BAWK_BAWK"));
        assert!(is_tag_name_valid("BAAAAAAAAAAAAWK"));
        assert!(is_tag_name_valid("BAAWK-BAWWK"));
        assert!(is_tag_name_valid("B"));
        assert!(is_tag_name_valid("BA"));
        assert!(is_tag_name_valid("BAW"));
        assert!(is_tag_name_valid("B4W"));
        assert!(is_tag_name_valid("B4Wk11"));
        assert!(is_tag_name_valid("11B4Wk"));
    }

    #[test]
    fn should_invalidate_incorrect_tags() {
        assert!(!is_tag_name_valid("bawk~"));
        assert!(!is_tag_name_valid("_BAWK_BAWK"));
        assert!(!is_tag_name_valid("BAWK_BAWK_"));
        assert!(!is_tag_name_valid("_BAWKBAWK_"));
        assert!(!is_tag_name_valid("BAWK BAWK bAWK BAWK"));
        assert!(!is_tag_name_valid("'BAWK"));
        assert!(!is_tag_name_valid("-BAWK"));
        assert!(!is_tag_name_valid("_BAWK"));
        assert!(!is_tag_name_valid("BAWK'"));
        assert!(!is_tag_name_valid("BAWK-"));
        assert!(!is_tag_name_valid("BAWK_"));
        assert!(!is_tag_name_valid("B=+[2{}AWK"));
        assert!(!is_tag_name_valid(""));
        assert!(!is_tag_name_valid("-"));
        assert!(!is_tag_name_valid("_"));
        assert!(!is_tag_name_valid("--"));
        assert!(!is_tag_name_valid("---"));
        assert!(!is_tag_name_valid("__"));
        assert!(!is_tag_name_valid("___"));
    }
}

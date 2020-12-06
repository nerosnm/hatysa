//! The react command converts an ASCII-alphanumeric string into a series of
//! reaction emojis, which it adds to a target message.

use tracing::{instrument, warn};

use std::collections::HashMap;

use super::{CommandError, Response};

#[instrument]
pub fn react(input: String) -> Result<Response, CommandError> {
    // Ignore spaces by removing them before checking if the input is valid.
    let input = input.replace(" ", "");

    // An input string is only valid if it is entirely composed of alphanumeric
    // characters, and if each one only appears once.
    let non_alphanum = input.contains(|c: char| !c.is_alphanumeric());
    let valid = !non_alphanum
        && input
            .chars()
            .fold(HashMap::new(), |mut acc, next| {
                *acc.entry(next.to_ascii_uppercase()).or_insert(0) += 1;
                acc
            })
            .values()
            .all(|&v| v == 1);

    if valid {
        let response = Response::React {
            reactions: to_reactions(&input),
        };

        Ok(response)
    } else if non_alphanum {
        warn!("string contains non-alphanumeric characters");

        Err(CommandError::NonAlphanumeric { original: input })
    } else {
        warn!("string contains repeated characters");

        Err(CommandError::Repetition { original: input })
    }
}

const VARIATION_SELECTOR_16: u32 = 0xfe0f;
const COMBINING_ENCLOSING_KEYCAP: u32 = 0x20e3;

/// Convert a string to a sequence of emojis representing its characters, using
/// regional indicators for alphabetic characters and keycap sequences for
/// numerals. Any non-ascii-alphanumeric characters are simply left as-is in the
/// output string.
fn to_reactions(input: &str) -> Vec<String> {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' => std::char::from_u32(c as u32 + 0x1f1a5)
                .unwrap_or(c)
                .to_string(),
            'a'..='z' => std::char::from_u32(c as u32 + 0x1f185)
                .unwrap_or(c)
                .to_string(),
            '0'..='9' => {
                // Create a keycap sequence by adding U+20E3 COMBINING ENCLOSING
                // KEYCAP after the numeral.
                let mut num = String::new();
                num.push(c);
                num.push(unsafe { std::char::from_u32_unchecked(VARIATION_SELECTOR_16) });
                num.push(unsafe { std::char::from_u32_unchecked(COMBINING_ENCLOSING_KEYCAP) });
                num
            }
            _ => c.to_string(),
        })
        .collect()
}

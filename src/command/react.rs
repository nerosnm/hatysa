//! The react command converts an ASCII-alphanumeric string into a series of
//! reaction emojis, which it adds to a target message.

use serenity::model::{
    channel::ReactionType,
    id::{ChannelId, MessageId},
};
use tracing::{instrument, warn};

use std::collections::HashMap;

use super::{CommandError, Response};

#[instrument]
pub fn react(
    channel_id: ChannelId,
    command_id: MessageId,
    target_id: MessageId,
    raw_reaction: String,
) -> Result<Vec<Response>, CommandError> {
    // Ignore spaces by removing them before checking if the input is valid.
    let raw_reaction = raw_reaction.replace(" ", "");

    // An input string is only valid if it is entirely composed of alphanumeric
    // characters, and if each one only appears once.
    let non_alphanum = raw_reaction.contains(|c: char| !c.is_alphanumeric());
    let valid = !non_alphanum
        && raw_reaction
            .chars()
            .fold(HashMap::new(), |mut acc, next| {
                *acc.entry(next.to_ascii_uppercase()).or_insert(0) += 1;
                acc
            })
            .values()
            .all(|&v| v == 1);

    if valid {
        let mut responses = vec![Response::DeleteMessage {
            channel_id,
            message_id: command_id,
        }];

        let mut reactions = to_reactions(&raw_reaction)
            .into_iter()
            .map(|reaction| Response::React {
                channel_id,
                message_id: target_id,
                reaction,
            })
            .collect::<Vec<_>>();

        responses.append(&mut reactions);

        Ok(responses)
    } else if non_alphanum {
        warn!("string contains non-alphanumeric characters");

        Err(CommandError::NonAlphanumeric {
            original: raw_reaction,
        })
    } else {
        warn!("string contains repeated characters");

        Err(CommandError::Repetition {
            original: raw_reaction,
        })
    }
}

const VARIATION_SELECTOR_16: u32 = 0xfe0f;
const COMBINING_ENCLOSING_KEYCAP: u32 = 0x20e3;

/// Convert a string to a sequence of reactions representing its characters,
/// using regional indicators for alphabetic characters and keycap sequences for
/// numerals. Any non-ascii-alphanumeric characters are simply left as-is in the
/// output string.
fn to_reactions(input: &str) -> Vec<ReactionType> {
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
        .map(ReactionType::Unicode)
        .collect()
}

//! Convert text to vaporwave (fullwidth) text.

use serenity::model::id::ChannelId;
use tracing::{debug, instrument};

use super::{CommandError, Response};

#[instrument]
pub fn vape(channel_id: ChannelId, input: String) -> Result<Vec<Response>, CommandError> {
    let vapified = vapify(input)?;

    debug!(?vapified);

    Ok(vec![Response::SendMessage {
        channel_id,
        message: vapified,
    }])
}

fn vapify(input: String) -> Result<String, CommandError> {
    input
        .chars()
        .map(|c| {
            let val = c as u32;
            match val {
                0x0020 => std::char::from_u32(0x3000).ok_or(CommandError::Internal(
                    "Invalid fullwidth space character".to_string(),
                )),
                0x0021..=0x007e => std::char::from_u32(val + 0xfee0).ok_or(CommandError::Internal(
                    "fullwidth character equivalent should be valid".to_string(),
                )),
                _ => Ok(c),
            }
        })
        .collect::<Result<String, CommandError>>()
}

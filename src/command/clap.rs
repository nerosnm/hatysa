//! Insert clapping emojis between every word of the input text.

use serenity::model::id::ChannelId;
use tracing::{debug, instrument};

use super::{CommandError, Response};

#[instrument]
pub fn clap(channel_id: ChannelId, input: String) -> Result<Vec<Response>, CommandError> {
    let mut words = input.split(" ");

    let clappified = words
        .next()
        .map(|first| {
            words.fold(first.to_string(), |mut acc, next| {
                acc.push_str(&format!(" 👏 {}", next));
                acc
            })
        })
        .unwrap_or_else(|| String::new());

    debug!("clappified response: {}", clappified);

    Ok(vec![Response::SendMessage {
        channel_id,
        message: clappified,
    }])
}

//! Convert text to Spongebob-case text.

use serenity::model::id::ChannelId;
use tracing::{debug, instrument};

use super::{CommandError, Response};

#[instrument]
pub fn spongebob(channel_id: ChannelId, input: String) -> Result<Vec<Response>, CommandError> {
    let (_, spongebobified) =
        input
            .chars()
            .fold((false, String::new()), |(upper, mut output), next_char| {
                let next_upper = if next_char.is_alphanumeric() {
                    if upper {
                        output.push(next_char.to_ascii_uppercase());
                    } else {
                        output.push(next_char.to_ascii_lowercase());
                    }

                    !upper
                } else {
                    output.push(next_char);
                    upper
                };

                (next_upper, output)
            });

    debug!("spongebobified response: {}", spongebobified);

    Ok(vec![Response::SendMessage {
        channel_id,
        message: spongebobified,
    }])
}

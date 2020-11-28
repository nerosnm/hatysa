//! The sketchify command converts a URL into a "sketchy" version using [the
//! Sketchify API][sketchify].
//!
//! [sketchify]: https://verylegit.link

use serenity::{
    model::id::{ChannelId, MessageId, UserId},
    utils::MessageBuilder,
};
use tracing::{error, instrument};
use url::Url;

use super::{CommandError, Response};

#[instrument]
pub fn sketchify(
    url: Url,
    channel_id: ChannelId,
    command_id: MessageId,
    author_id: UserId,
) -> Result<Vec<Response>, CommandError> {
    let api_params = [("long_url", url.to_string())];
    let client = reqwest::Client::new();
    let mut res = client
        .post("http://verylegit.link/sketchify")
        .form(&api_params)
        .send()
        .map_err(|err| {
            error!("failed to send request");
            err
        })?;

    let sketchified_url_str = res.text().map_err(|err| {
        error!("failed to extract text from API response");
        err
    })?;

    let sketchified_url = if !sketchified_url_str.starts_with("http") {
        Url::parse(&*format!("http://{}", sketchified_url_str))
    } else {
        Url::parse(&*sketchified_url_str)
    }
    .map_err(|err| {
        error!("failed to parse returned URL");
        err
    })?;

    let message = MessageBuilder::new()
        .mention(&author_id)
        .push(": <")
        .push(sketchified_url)
        .push(">")
        .build();

    Ok(vec![
        Response::SendMessage {
            channel_id,
            message,
        },
        Response::DeleteMessage {
            channel_id,
            message_id: command_id,
        },
    ])
}

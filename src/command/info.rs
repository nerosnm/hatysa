//! Provide some info about the currently running instance of the bot.

use serenity::model::id::ChannelId;
use tracing::{debug, instrument};

use super::Response;
use crate::VERSION;

#[instrument]
pub fn info(channel_id: ChannelId) -> Vec<Response> {
    debug!(version = VERSION);

    vec![Response::SendMessage {
        channel_id,
        message: format!("version: {}", VERSION),
    }]
}

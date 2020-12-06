//! Provide some info about the currently running instance of the bot.

use serenity::{builder::CreateEmbed, model::id::ChannelId};
use tracing::{debug, instrument};

use super::Response;
use crate::VERSION;

#[instrument]
pub fn info(channel_id: ChannelId) -> Vec<Response> {
    let uptime = "0h 0m 0s";
    let homepage = "https://sr.ht/~nerosnm/hatysa";

    debug!(version = VERSION, ?uptime, ?homepage);

    let embed = CreateEmbed::default()
        .field("Version", VERSION, true)
        .field("Uptime", uptime, true)
        .field("Homepage", homepage, false)
        .to_owned();

    vec![Response::SendEmbed { channel_id, embed }]
}

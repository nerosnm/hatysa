//! Provide some info about the currently running instance of the bot.

use chrono::Utc;
use serenity::{builder::CreateEmbed, model::id::ChannelId};
use tracing::{debug, instrument};

use super::Response;
use crate::{START_TIME, VERSION};

#[instrument]
pub async fn info(channel_id: ChannelId) -> Vec<Response> {
    let start_time = *START_TIME.lock().await;
    let now = Utc::now();
    let uptime = now - start_time;

    let uptime_str = format!(
        "{}d {}h {}m {}s",
        uptime.num_days(),
        uptime.num_hours(),
        uptime.num_minutes(),
        uptime.num_seconds()
    );

    let homepage = "https://sr.ht/~nerosnm/hatysa";

    debug!(version = VERSION, ?start_time, ?uptime_str, ?homepage);

    let embed = CreateEmbed::default()
        .field("Version", VERSION, true)
        .field("Uptime", uptime_str, true)
        .field("Homepage", homepage, false)
        .to_owned();

    vec![Response::SendEmbed { channel_id, embed }]
}

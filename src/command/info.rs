//! Provide some info about the currently running instance of the bot.

use chrono::{DateTime, Duration, Utc};
use serenity::{builder::CreateEmbed, model::id::ChannelId};
use tracing::{debug, instrument};

use super::Response;
use crate::VERSION;

#[instrument]
pub async fn info(channel_id: ChannelId, start_time: DateTime<Utc>) -> Vec<Response> {
    // Get the time this instance started running.
    let now = Utc::now();
    let mut uptime = now - start_time;

    // Convert the uptime into days, hours, minutes and seconds.
    let days = uptime.num_days();
    uptime = uptime - Duration::days(days);

    let hours = uptime.num_hours();
    uptime = uptime - Duration::hours(hours);

    let minutes = uptime.num_minutes();
    uptime = uptime - Duration::minutes(minutes);

    let seconds = uptime.num_seconds();

    let version = VERSION;
    let uptime_str = format!("{}d {}h {}m {}s", days, hours, minutes, seconds);
    let homepage = "https://sr.ht/~nerosnm/hatysa";

    debug!(?version, ?start_time, ?uptime_str, ?homepage);

    let embed = CreateEmbed::default()
        .field("Version", version, true)
        .field("Uptime", uptime_str, true)
        .field("Homepage", homepage, false)
        .to_owned();

    vec![Response::SendEmbed { channel_id, embed }]
}

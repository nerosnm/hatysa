//! Provide some info about the currently running instance of the bot.

use chrono::{DateTime, Duration, Utc};

use super::Response;
use crate::VERSION;

#[instrument]
pub async fn info(start_time: DateTime<Utc>) -> Response {
    // Get the time this instance started running.
    let now = Utc::now();
    let mut uptime = now - start_time;

    debug!(?now, ?uptime);

    // Convert the uptime into days, hours, minutes and seconds.
    let days = uptime.num_days();
    uptime = uptime - Duration::days(days);

    let hours = uptime.num_hours();
    uptime = uptime - Duration::hours(hours);

    let minutes = uptime.num_minutes();
    uptime = uptime - Duration::minutes(minutes);

    let seconds = uptime.num_seconds();

    let response = Response::Info {
        version: VERSION.to_string(),
        uptime: (days, hours, minutes, seconds),
        homepage: "https://github.com/nerosnm/hatysa".to_string(),
    };

    debug!(?response);

    response
}

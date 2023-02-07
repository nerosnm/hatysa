//! Hatysa is a Discord bot that implements a few fun commands.
//!
//! ## Usage
//!
//! To run the bot, you'll need to provide a Discord token (obtainable from the
//! [Discord Developer Portal](https://discord.com/developers)), as follows:
//!
//! ```bash
//! $ DISCORD_TOKEN="<token>" cargo run
//! ```
//!
//! The prefix can be changed from the default (`,`) using `HATYSA_PREFIX`, and
//! you might also want to [change the tracing subscriber filter][sub] to
//! customise what log messages are printed out:
//!
//! [sub]:
//! ../tracing_subscriber/fmt/index.html#filtering-events-with-environment-variables
//!
//! ```bash
//! $ DISCORD_TOKEN="<token>" HATYSA_PREFIX="!" RUST_LOG="info,hatysa=debug" cargo run
//! ```

#[macro_use]
extern crate tracing;

pub mod handler;
pub mod task;

use chrono::Utc;
use eyre::{Result, WrapErr};
use serenity::prelude::*;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use std::env;

use handler::Handler;

#[tokio::main]
async fn main() -> Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,hatysa=debug"));
    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("global default subscriber should have been set");

    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").wrap_err("expected a token in the environment")?;
    let prefix = env::var("HATYSA_PREFIX").unwrap_or_else(|_| ",".to_string());

    let start_time = Utc::now();
    info!("starting hatysa at {}", start_time);

    let mut client = Client::builder(&token, GatewayIntents::default())
        .event_handler(Handler { prefix, start_time })
        .await?;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}

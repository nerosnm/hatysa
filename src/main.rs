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
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("global default subscriber should have been set");

    let token = env::var("DISCORD_TOKEN").wrap_err("expected a token in the environment")?;
    let prefix = env::var("HATYSA_PREFIX").unwrap_or(",".to_string());

    let start_time = Utc::now();
    info!("starting hatysa at {}", start_time);

    let mut client = Client::builder(&token)
        .event_handler(Handler { prefix, start_time })
        .await?;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}

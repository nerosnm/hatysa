pub mod command;
pub mod handler;

use eyre::{Result, WrapErr};
use serenity::prelude::*;
use tracing::error;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use std::env;

use handler::Handler;

const VERSION: &str = "0.2.1";

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("global default subscriber should have been set");

    let token = env::var("DISCORD_TOKEN").wrap_err("expected a token in the environment")?;
    let prefix = env::var("HATYSA_PREFIX").unwrap_or(",".to_string());

    let mut client = Client::builder(&token)
        .event_handler(Handler { prefix })
        .await?;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}

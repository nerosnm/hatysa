pub mod command;
pub mod handler;

use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use serenity::prelude::*;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use std::env;
use std::sync::Arc;

use handler::Handler;

const VERSION: &str = "0.2.1";

lazy_static::lazy_static! {
    static ref START_TIME: Arc<Mutex<DateTime<Utc>>> = Arc::new(Mutex::new(Utc::now()));
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("global default subscriber should have been set");

    let token = env::var("DISCORD_TOKEN").wrap_err("expected a token in the environment")?;
    let prefix = env::var("HATYSA_PREFIX").unwrap_or(",".to_string());

    {
        let start_time = START_TIME.lock().await;
        info!("starting hatysa at {}", start_time);
    }

    let mut client = Client::builder(&token)
        .event_handler(Handler { prefix })
        .await?;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}

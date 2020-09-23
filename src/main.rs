use serenity::{
    async_trait,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult, StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
    prelude::*,
    utils::MessageBuilder,
};
use url::Url;

use std::env;

#[command]
#[num_args(0)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let channel = msg.channel_id.to_channel(&ctx).await?;

    let response = MessageBuilder::new()
        .push("User ")
        .push_bold_safe(&msg.author.name)
        .push(" used the 'ping' command in the ")
        .mention(&channel)
        .push(" channel")
        .build();

    msg.channel_id.say(&ctx.http, &response).await?;

    Ok(())
}

#[command]
#[num_args(1)]
async fn sketchify(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let url_str = args.parse::<String>()?;
    let url = Url::parse(&*url_str).or_else(|_| Url::parse(&*format!("http://{}", url_str)))?;

    let api_params = [("long_url", url.to_string())];
    let client = reqwest::Client::new();
    let mut res = client
        .post("http://verylegit.link/sketchify")
        .form(&api_params)
        .send()?;

    let sketchified_url_str = if res.status().is_success() {
        res.text()
            .map_err(|err| format!("Failed to parse API response: {}", err))
    } else {
        Err(format!("API call failed, response code: {}", res.status()))
    }?;

    let sketchified_url = if !sketchified_url_str.starts_with("http") {
        Url::parse(&*format!("http://{}", sketchified_url_str))
    } else {
        Url::parse(&*sketchified_url_str)
    }?;

    let response = MessageBuilder::new().push(sketchified_url).build();
    msg.channel_id.say(&ctx.http, &response).await?;

    Ok(())
}

#[group]
#[commands(ping, sketchify)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(","))
        .group(&GENERAL_GROUP);

    let mut client = Client::new(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}

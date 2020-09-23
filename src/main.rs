use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    utils::MessageBuilder,
};
use url::Url;

use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.content == ",ping" {
            let channel = match msg.channel_id.to_channel(&context).await {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);

                    return;
                }
            };

            let response = MessageBuilder::new()
                .push("User ")
                .push_bold_safe(&msg.author.name)
                .push(" used the 'ping' command in the ")
                .mention(&channel)
                .push(" channel")
                .build();

            if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content.starts_with(",sketchify") {
            let url_str = msg.content.strip_prefix(",sketchify").map(str::trim);

            if let None = url_str {
                println!("Error stripping prefix from URL.");
                return;
            }

            let params = [("long_url", url_str.unwrap())];
            let client = reqwest::Client::new();
            let res = client
                .post("http://verylegit.link/sketchify")
                .form(&params)
                .send();

            let response = match res.and_then(|mut res| res.text()) {
                Err(err) => MessageBuilder::new()
                    .push_bold("Error")
                    .push(" sketchifying link: ")
                    .push_mono_safe(err)
                    .build(),
                Ok(res) => MessageBuilder::new().push_safe(res).build(),
            };

            if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content.starts_with(",echo") {
            let url_res = msg
                .content
                .strip_prefix(",echo")
                .map(str::trim)
                .ok_or("Error extracting URL from message".to_string())
                .and_then(|url_str| {
                    Url::parse(url_str).map_err(|e| format!("Unable to parse URL: {}", e))
                });

            let response = match url_res {
                Ok(url) => MessageBuilder::new().push(url).build(),
                Err(err) => MessageBuilder::new().push_bold("Error: ").push(err).build(),
            };

            if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::new(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}

use serenity::{
    async_trait,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult, StandardFramework,
    },
    model::{
        channel::{Message, ReactionType},
        gateway::Activity,
        gateway::Ready,
    },
    prelude::*,
    utils::MessageBuilder,
};
use url::Url;

use std::collections::HashMap;
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

    let response = MessageBuilder::new()
        .mention(&msg.author)
        .push(": <")
        .push(sketchified_url)
        .push(">")
        .build();
    msg.channel_id.say(&ctx.http, &response).await?;

    msg.delete(&ctx.http).await?;

    Ok(())
}

#[command]
#[num_args(1)]
async fn react(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let input_str = args.parse::<String>()?;

    // An input string is only valid if it is entirely composed of alphanumeric
    // characters, and if each one only appears once.
    let non_alphanum = input_str.contains(|c: char| !c.is_alphanumeric());
    let valid = !non_alphanum
        && input_str
            .chars()
            .fold(HashMap::new(), |mut acc, next| {
                *acc.entry(next.to_ascii_uppercase()).or_insert(0) += 1;
                acc
            })
            .values()
            .all(|&v| v == 1);

    if valid {
        if let Some(prev) = msg
            .channel_id
            .messages(&ctx.http, |retriever| retriever.before(msg.id).limit(1))
            .await?
            .first()
        {
            for c in to_reactions(&input_str) {
                prev.react(&ctx.http, c).await?;
            }
        }

        msg.delete(&ctx.http).await?;
    } else if non_alphanum {
        let response = MessageBuilder::new()
            .push("String ")
            .push_bold_safe(input_str.to_ascii_uppercase())
            .push(" contains non-alphanumeric characters!")
            .build();

        msg.channel_id.say(&ctx.http, &response).await?;
    } else {
        let response = MessageBuilder::new()
            .push("String ")
            .push_bold_safe(input_str.to_ascii_uppercase())
            .push(" contains repeated characters!")
            .build();

        msg.channel_id.say(&ctx.http, &response).await?;
    }

    Ok(())
}

const VARIATION_SELECTOR_16: u32 = 0xfe0f;
const COMBINING_ENCLOSING_KEYCAP: u32 = 0x20e3;

/// Convert a string to a sequence of reactions representing its characters,
/// using regional indicators for alphabetic characters and keycap sequences for
/// numerals. Any non-ascii-alphanumeric characters are simply left
/// as-is in the output string.
fn to_reactions(input: &str) -> Vec<ReactionType> {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' => std::char::from_u32(c as u32 + 0x1f1a5)
                .unwrap_or(c)
                .to_string(),
            'a'..='z' => std::char::from_u32(c as u32 + 0x1f185)
                .unwrap_or(c)
                .to_string(),
            '0'..='9' => {
                // Create a keycap sequence by adding U+20E3 COMBINING ENCLOSING
                // KEYCAP after the numeral.
                let mut num = String::new();
                num.push(c);
                num.push(unsafe { std::char::from_u32_unchecked(VARIATION_SELECTOR_16) });
                num.push(unsafe { std::char::from_u32_unchecked(COMBINING_ENCLOSING_KEYCAP) });
                num
            }
            _ => c.to_string(),
        })
        .map(ReactionType::Unicode)
        .collect()
}

#[group]
#[commands(ping, sketchify, react)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        ctx.set_activity(Activity::playing(",react")).await;
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

//! The ping command, which just replies to the user who sent the command.

use serenity::{
    model::id::{ChannelId, UserId},
    utils::MessageBuilder,
};

use super::Response;

pub fn ping(channel_id: ChannelId, author_id: UserId) -> Vec<Response> {
    let reply = MessageBuilder::new()
        .push("User ")
        .mention(&author_id)
        .push(" used the 'ping' command in the ")
        .mention(&channel_id)
        .push(" channel")
        .build();

    vec![
        Response::SendMessage {
            channel_id,
            message: "Ping!".to_string(),
        },
        Response::SendMessage {
            channel_id,
            message: reply,
        },
    ]
}

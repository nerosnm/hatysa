//! Execute commands and return their output.
//!
//! The command layer takes commands from the [handler layer][handler] and
//! determines the appropriate action, if any, to take based on the result of
//! those commands, which it returns back to the handler.
//!
//! [handler]: ../handler/index.html

mod ping;
mod react;
mod sketchify;
mod spongebob;
mod zalgo;

use serenity::{
    model::{
        channel::ReactionType,
        id::{ChannelId, MessageId, UserId},
    },
    utils::MessageBuilder,
};
use thiserror::Error;
use url::ParseError;

/// Commands that can be performed.
pub enum Command {
    /// A request from a user for a response from the bot, for testing purposes.
    Ping {
        /// The ID of the channel the ping request was sent in.
        channel_id: ChannelId,
        /// The id of the user who sent the ping request.
        author_id: UserId,
    },
    /// Add a string as a series of reactions to a target message.
    React {
        /// The ID of the channel the command was sent in.
        channel_id: ChannelId,
        /// The ID of the message that contained the reaction command.
        command_id: MessageId,
        /// The ID of the message that should be reacted to.
        target_id: MessageId,
        /// The string to react to the target with.
        reaction: String,
    },
    /// Convert a URL to a "sketchified" equivalent using [the Sketchify
    /// API][sketchify].
    ///
    /// [sketchify]: https://verylegit.link
    Sketchify {
        /// The string provided for the URL to sketchify.
        url_raw: String,
        /// The ID of the channel the sketchify request was sent in.
        channel_id: ChannelId,
        /// The ID of the message that contained the sketchify command.
        command_id: MessageId,
        /// The ID of the user who sent the sketchify request.
        author_id: UserId,
    },
    /// Convert text to Spongebob-case text.
    Spongebob {
        /// The ID of the channel the request was sent in.
        channel_id: ChannelId,
        /// The input to convert.
        input: String,
    },
    /// Convert text to Zalgo text.
    Zalgo {
        /// The ID of the channel the request was sent in.
        channel_id: ChannelId,
        /// The input to convert.
        input: String,
        /// If provided, the maximum number of characters to output.
        max_chars: Option<usize>,
    },
}

impl Command {
    /// Execute a command, returning the responses that should be performed (if any)
    /// on success.
    pub async fn execute(self) -> Result<Vec<Response>, CommandError> {
        match self {
            Command::Ping {
                channel_id,
                author_id,
            } => Ok(ping::ping(channel_id, author_id)),
            Command::React {
                channel_id,
                command_id,
                target_id,
                reaction,
            } => react::react(channel_id, command_id, target_id, reaction),
            Command::Sketchify {
                url_raw,
                channel_id,
                command_id,
                author_id,
            } => sketchify::sketchify(url_raw, channel_id, command_id, author_id),
            Command::Spongebob { channel_id, input } => spongebob::spongebob(channel_id, input),
            Command::Zalgo {
                channel_id,
                input,
                max_chars,
            } => zalgo::zalgo(channel_id, input, max_chars),
        }
    }
}

/// Possible responses as a result of a command.
pub enum Response {
    /// Respond with a message in a channel.
    SendMessage {
        /// The ID of the channel the message should be sent in.
        channel_id: ChannelId,
        /// The message to send.
        message: String,
    },
    /// React to a message.
    React {
        /// The ID of the channel the message to react to is in.
        channel_id: ChannelId,
        /// The ID of the message that the reaction should be added to.
        message_id: MessageId,
        /// The reaction to add to the message.
        reaction: ReactionType,
    },
    /// Delete a message.
    DeleteMessage {
        /// The ID of the channel the message to delete is in.
        channel_id: ChannelId,
        /// The ID of the message that should be deleted.
        message_id: MessageId,
    },
}

/// Errors that could occur during command processing.
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("string \"{}\" contains non-alphanumeric characters", original)]
    NonAlphanumeric { original: String },
    #[error("string \"{}\" contains repeated characters", original)]
    Repetition { original: String },
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] ParseError),
    #[error("could not complete request: {0}")]
    Request(#[from] reqwest::Error),
}

impl CommandError {
    /// Return a user friendly version of the error, suitable for sending to the
    /// user as a Discord message.
    pub fn user_friendly_message(&self) -> String {
        match self {
            CommandError::NonAlphanumeric { original } => MessageBuilder::new()
                .push("String ")
                .push_bold(original.to_uppercase())
                .push(" contains non-alphanumeric characters!")
                .build(),
            CommandError::Repetition { original } => MessageBuilder::new()
                .push("String ")
                .push_bold(original.to_uppercase())
                .push(" contains repeated characters!")
                .build(),
            CommandError::InvalidUrl(_) => MessageBuilder::new().push("Invalid URL!").build(),
            CommandError::Request(_) => MessageBuilder::new()
                .push("Failed to complete request. Please try again.")
                .build(),
        }
    }
}

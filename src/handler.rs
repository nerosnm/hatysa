//! Handle incoming events from Discord.
//!
//! The handler layer takes [`Context`][ctx] and [`Message`][msg] information as
//! input, and outputs information by:
//!
//! - First attempting to write response information back to Discord.
//!     - If that fails, writing information about the communication failure to
//!     the console.
//! - If the response information was a failure, writing response information to
//! the console.
//!
//! What response information should actually be written is determined by
//! parsing the input to determine its intent, and in the event of the input
//! forming a command, passing that command to [`command::execute()`][execute].
//!
//! [ctx]: ../../serenity/client/struct.Context.html
//! [msg]: ../../serenity/model/channel/struct.Message.html
//! [handle]: ../command/fn.execute.html

use eyre::{Result, WrapErr};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Activity, gateway::Ready, id::ChannelId, id::MessageId},
    prelude::*,
};
use thiserror::Error;
use tracing::{error, info, warn};

use crate::command::{execute, Command, CommandError, Response};

pub struct Handler {
    pub prefix: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        ctx.set_activity(Activity::playing(&*format!(",react")))
            .await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if let Some(command_result) = self.interpret_command(&ctx, &msg).await {
            info!("message id={} is a command", msg.id);

            match command_result {
                Ok(command) => {
                    info!("command in message id={} is valid, executing", msg.id);

                    match execute(command).await {
                        Ok(responses) => {
                            info!(
                                "successfully executed command in message id={}, running responses",
                                msg.id
                            );

                            for response in responses {
                                if let Err(err) = self.respond(&ctx, response).await {
                                    error!("{:#}", err);
                                }
                            }
                        }
                        Err(err) => {
                            info!(
                                "failed to execute command in message id={}, reporting to user",
                                msg.id
                            );

                            self.report_to_user(&ctx, msg.channel_id, err).await;
                        }
                    }
                }
                Err(err) => {
                    error!("command in message id={} is invalid: {:#}", msg.id, err);
                }
            }
        } else {
            info!("message id={} is not a command", msg.id);
        }
    }
}

impl Handler {
    async fn interpret_command(&self, ctx: &Context, msg: &Message) -> Option<Result<Command>> {
        if let Some(tail) = msg.content.strip_prefix(&self.prefix) {
            if tail.starts_with("ping") {
                Some(Ok(Command::Ping {
                    channel_id: msg.channel_id,
                    author_id: msg.author.id,
                }))
            } else if let Some(tail) = tail.strip_prefix("react").map(|tail| tail.trim()) {
                Some(
                    self.find_previous_id(ctx, msg)
                        .await
                        .map(|prev_id| Command::React {
                            channel_id: msg.channel_id,
                            command_id: msg.id,
                            target_id: prev_id,
                            reaction: tail.to_owned(),
                        }),
                )
            } else if let Some(tail) = tail.strip_prefix("sketchify").map(|tail| tail.trim()) {
                Some(Ok(Command::Sketchify {
                    url_raw: tail.to_owned(),
                    channel_id: msg.channel_id,
                    command_id: msg.id,
                    author_id: msg.author.id,
                }))
            } else {
                None
            }
        } else {
            None
        }
    }

    async fn find_previous_id(&self, ctx: &Context, msg: &Message) -> Result<MessageId> {
        let prev = msg
            .channel_id
            .messages(&ctx.http, |retriever| retriever.before(msg.id).limit(1))
            .await
            .wrap_err(HandlerError::GetPrevious { message_id: msg.id })?;

        let target = prev
            .first()
            .ok_or(HandlerError::GetPrevious { message_id: msg.id })?;

        Ok(target.id)
    }

    async fn respond(&self, ctx: &Context, response: Response) -> Result<()> {
        match response {
            Response::SendMessage {
                channel_id,
                message,
            } => {
                // Send the reply.
                channel_id
                    .say(&ctx.http, message)
                    .await
                    .wrap_err(HandlerError::SendMessage)?;
            }
            Response::React {
                channel_id,
                message_id,
                reaction,
            } => {
                // Get the message from the channel so we can react to it.
                let message = channel_id
                    .message(&ctx.http, message_id)
                    .await
                    .wrap_err(HandlerError::GetMessage { message_id })
                    .wrap_err(HandlerError::React { message_id })?;

                // React to the message.
                message
                    .react(&ctx.http, reaction)
                    .await
                    .wrap_err(HandlerError::React { message_id })?;
            }
            Response::DeleteMessage {
                channel_id,
                message_id,
            } => {
                // Get the message from the channel so we can delete it.
                let message = channel_id
                    .message(&ctx.http, message_id)
                    .await
                    .wrap_err(HandlerError::GetMessage { message_id })
                    .wrap_err(HandlerError::Delete { message_id })?;

                // Delete the message.
                message
                    .delete(&ctx.http)
                    .await
                    .wrap_err(HandlerError::Delete { message_id })?;
            }
        }

        Ok(())
    }

    async fn report_to_user(&self, ctx: &Context, channel_id: ChannelId, err: CommandError) {
        warn!(
            "reporting error to user in channel id={}: {:#}",
            channel_id, err
        );

        match channel_id.say(&ctx.http, err.user_friendly_message()).await {
            Ok(_) => info!("successfully reported error"),
            Err(err) => error!("unable to report error: {:#}", err),
        }
    }
}

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("unable to send message")]
    SendMessage,
    #[error("unable to react to message {}", message_id)]
    React { message_id: MessageId },
    #[error("unable to get message by id {}", message_id)]
    GetMessage { message_id: MessageId },
    #[error("unable to get message before message with id {}", message_id)]
    GetPrevious { message_id: MessageId },
    #[error("unable to delete message {}", message_id)]
    Delete { message_id: MessageId },
}
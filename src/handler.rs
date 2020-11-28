//! Handle incoming events from Discord.
//!
//! The handler layer takes [`Context`][ctx] and [`Message`][msg] information as
//! input, and outputs information by:
//!
//! - First attempting to write response information back to Discord, including
//! user-facing errors.
//!     - If that fails, logging information about the communication failure.
//! - If non-user-facing errors occurred, logging them.
//!
//! What response information should actually be written is determined by
//! parsing the input to determine its intent, and in the event of the input
//! forming a command, running it with [`execute()`][execute].
//!
//! [ctx]: ../../serenity/client/struct.Context.html
//! [msg]: ../../serenity/model/channel/struct.Message.html
//! [execute]: ../command/enum.Command.html#method.execute

use eyre::{Result, WrapErr};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::{Message, ReactionType},
        gateway::Activity,
        gateway::Ready,
        id::{ChannelId, MessageId},
    },
};
use structopt::{clap::Error as ClapError, StructOpt};
use thiserror::Error;
use tracing::{error, info, warn};

use crate::command::{Command, CommandError, Response};
use crate::parse::Options;

/// Hatysa event handler.
///
/// This is the outermost entrypoint for command execution. Messages passed to
/// [`message()`][message] are parsed to determine if they match a command, and
/// if they do, the parsed command is handed to [`command::execute()`][execute].
///
/// [message]: #method.message
/// [execute]: ../command/fn.execute.html
pub struct Handler {
    /// The string that must come before all commands' names.
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

                    match command.execute().await {
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

                            self.report_command_error(&ctx, msg, err).await;
                        }
                    }
                }
                Err(err) => {
                    error!(
                        "command in message id={} is invalid, reporting to user",
                        msg.id
                    );

                    self.report_clap_error(&ctx, msg.channel_id, err).await;
                }
            }
        } else {
            info!("message id={} is not a command", msg.id);
        }
    }
}

impl Handler {
    /// Attempt to parse a message as a command, and gather all information
    /// needed to execute the command if parsing succeeds.
    ///
    /// If the message does not contain a command, `None` is returned. If the
    /// message does contain a command but it could not be parsed or prepared
    /// properly, `Some(Err(..))` is returned.
    async fn interpret_command(
        &self,
        ctx: &Context,
        msg: &Message,
    ) -> Option<Result<Command, ClapError>> {
        // Non-private messages must have a prefix on them, but it's optional
        // for private messages, so if we don't find a prefix, check if it was a
        // private message and allow it if it was.
        let tail = msg.content.strip_prefix(&self.prefix).or_else(|| {
            if msg.is_private() {
                Some(&*msg.content)
            } else {
                None
            }
        });

        if let Some(tail) = tail {
            match Options::from_iter_safe(tail.split_whitespace()) {
                Ok(Options::Clap { input }) => Some(Ok(Command::Clap {
                    channel_id: msg.channel_id,
                    input,
                })),
                Ok(Options::Info) => Some(Ok(Command::Info {
                    channel_id: msg.channel_id,
                })),
                Ok(Options::Ping) => Some(Ok(Command::Ping {
                    channel_id: msg.channel_id,
                    author_id: msg.author.id,
                })),
                Ok(Options::React { reaction }) => Some(self.find_previous_id(ctx, msg).await.map(
                    |prev_id| Command::React {
                        channel_id: msg.channel_id,
                        command_id: msg.id,
                        target_id: prev_id,
                        reaction,
                    },
                )),
                Ok(Options::Sketchify { url }) => Some(Ok(Command::Sketchify {
                    url,
                    channel_id: msg.channel_id,
                    command_id: msg.id,
                    author_id: msg.author.id,
                })),
                Ok(Options::Spongebob { input }) => Some(Ok(Command::Spongebob {
                    channel_id: msg.channel_id,
                    input,
                })),
                Ok(Options::Vape { input }) => Some(Ok(Command::Vape {
                    channel_id: msg.channel_id,
                    input,
                })),
                Ok(Options::Zalgo { input }) => Some(Ok(Command::Zalgo {
                    channel_id: msg.channel_id,
                    input: input.join(" "),
                    max_chars: None,
                })),
                Err(err) => {
                    self.report_clap_error(ctx, msg.channel_id, err);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Find the ID of the message that occurred immediately before `msg`.
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

    /// Carry out the given `response`.
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
            Response::SendEmbed {
                channel_id,
                mut embed,
            } => {
                let avatar_url = if let Ok(current_user) = ctx.http.get_current_user().await {
                    current_user.avatar_url()
                } else {
                    warn!("unable to retrieve avatar url for bot");
                    None
                };

                // Send a message with the embed in it.
                channel_id
                    .send_message(&ctx.http, |m| {
                        // Set up branding on embed.
                        embed
                            .author(|a| {
                                if let Some(url) = avatar_url {
                                    a.icon_url(url);
                                }

                                a.name("Hatysa").url("https://sr.ht/~nerosnm/hatysa")
                            })
                            .colour((244, 234, 62));

                        m.set_embed(embed)
                    })
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

    /// Report a [`CommandError`] to the user, by sending a message containing
    /// its [`err.user_friendly_message()`][ufm] .
    ///
    /// If the message reporting the error cannot be sent, the failure is logged
    /// at the [`error!()`][error] level.
    ///
    /// [ufm]: ../command/enum.CommandError.html#method.user_friendly_message
    /// [error]: ../../tracing/macro.error.html
    async fn report_command_error(&self, ctx: &Context, original: Message, err: CommandError) {
        warn!(
            "reporting error to user in channel id={}: {:#}",
            original.channel_id, err
        );

        let avatar_url = if let Ok(current_user) = ctx.http.get_current_user().await {
            current_user.avatar_url()
        } else {
            warn!("unable to retrieve avatar url for bot");
            None
        };

        match original
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.author(|a| {
                        if let Some(url) = avatar_url {
                            a.icon_url(url);
                        }

                        a.name("Hatysa").url("https://todo.sr.ht/~nerosnm/hatysa")
                    })
                    .field("Error", err.user_friendly_message(), true)
                    .footer(|f| {
                        // TODO: Implement ,help command
                        // f.text(format!(
                        //     "For help, run {prefix}help. Click OK to delete.",
                        //     prefix = self.prefix
                        // ))
                        f.text("Click OK to delete.")
                    })
                    .colour((244, 234, 62))
                })
                .reactions(vec![ReactionType::Unicode("🆗".to_string())])
            })
            .await
        {
            Ok(sent_message) => {
                info!("successfully reported error");

                if let Some(_) = sent_message
                    .await_reaction(&ctx)
                    .filter(|react| react.emoji == ReactionType::Unicode("🆗".to_string()))
                    .author_id(original.author.id)
                    .await
                {
                    info!(
                        "got an OK reaction on error message {}, deleting",
                        sent_message.id
                    );

                    match sent_message.delete(&ctx.http).await {
                        Ok(_) => {
                            info!("successfully deleted error message {}", sent_message.id);
                        }
                        Err(_) => {
                            error!("unable to delete error message {}", sent_message.id);
                        }
                    }

                    match original.delete(&ctx.http).await {
                        Ok(_) => {
                            info!("successfully deleted original message {}", sent_message.id);
                        }
                        Err(_) => {
                            error!("unable to delete original message {}", sent_message.id);
                        }
                    }
                }
            }
            Err(err) => error!("unable to report error: {:#}", err),
        }
    }

    /// Report a [`clap::Error`][claperror] to the user, by sending a message.
    ///
    /// If the message reporting the error cannot be sent, the failure is logged
    /// at the [`error!()`][error] level.
    ///
    /// [claperror]: ../../clap/struct.Error.html
    /// [error]: ../../tracing/macro.error.html
    async fn report_clap_error(&self, ctx: &Context, channel_id: ChannelId, err: ClapError) {
        warn!("reporting clap error to user in channel id={}", channel_id);

        match channel_id.say(&ctx.http, err.message).await {
            Ok(_) => info!("successfully reported error"),
            Err(err) => error!("unable to report error: {:#}", err),
        }
    }
}

/// Errors that could occur while handling a message or running commands as a
/// result.
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

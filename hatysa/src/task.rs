//! Tasks represent requests extracted from Discord messages, including both a
//! command and the context surrounding it.
//!
//! Tasks can be executed, which executes the underlying command and then
//! responds using the Discord bot, making use of the context to determine how
//! to respond.

use serenity::{
    builder::CreateEmbed,
    client::Context as ClientContext,
    model::{
        channel::{Message, ReactionType},
        id::MessageId,
    },
    utils::MessageBuilder,
};

use std::time::Duration;

use iota_orionis::command::{Command, CommandError, Response};

/// A task containing a command and context about the message that triggered the
/// command.
pub struct Task {
    /// The underlying command.
    command: Command,
    /// The context of the command.
    context: Context,
}

impl Task {
    /// Create a new task from a parsed command, the message that triggered it,
    /// and the context of the message.
    pub fn new(command: Command, ctx: ClientContext, message: Message) -> Self {
        Self {
            command,
            context: Context { ctx, message },
        }
    }

    /// Execute the task, including executing the underlying command and sending
    /// responses to the user.
    ///
    /// If any step in the process fails, an error will be returned.
    #[instrument(skip(self), fields(id = self.context.message.id.0))]
    pub async fn execute(self) {
        // First try to execute the command.
        match self.command.execute().await {
            Ok(response) => {
                // If execute() succeeded, then the command was valid and we
                // have some info to send back to the user.
                info!("successfully executed command, running responses");

                if let Err(err) = self.context.respond(response).await {
                    // If responding failed, then we should log what went wrong
                    // to the console.
                    error!("{}", err);
                } else {
                    info!("successfully responded");
                }
            }
            Err(err) => {
                // If execute() failed, then the command was invalid in some way
                // and we should report that to the user.
                info!("failed to execute command, reporting to user");

                if let Err(err) = self.context.report(err).await {
                    // If reporting the error to the user failed, then we should
                    // log what went wrong to the console.
                    error!("{}", err);
                } else {
                    info!("successfully reported error");
                }
            }
        }
    }
}

/// The context of a command.
struct Context {
    /// The context of the original event that triggered this task.
    ctx: ClientContext,
    /// The message that triggered the command.
    message: Message,
}

impl Context {
    /// Attempt to respond to the user with the result of a command.
    #[instrument(skip(self))]
    async fn respond(&self, response: Response) -> Result<(), TaskError> {
        match response {
            Response::Clap { output }
            | Response::Spongebob { output }
            | Response::Wavy { output }
            | Response::Zalgo { output } => {
                debug!("sending output in a plain message");

                self.message.channel_id.say(&self.ctx.http, output).await?;
            }
            Response::Info {
                version,
                uptime: (days, hours, minutes, seconds),
                homepage,
            } => {
                debug!("attempting to retrieve URL of bot user's avatar");

                let avatar_url = if let Ok(current_user) = self.ctx.http.get_current_user().await {
                    debug!("found avatar URL");
                    current_user.avatar_url()
                } else {
                    warn!("unable to retrieve avatar url for bot");
                    None
                };

                self.message
                    .channel_id
                    .send_message(&self.ctx.http, |m| {
                        debug!("constructing embed");

                        let mut embed = CreateEmbed::default();

                        embed
                            .author(|a| {
                                avatar_url.map(|url| a.icon_url(url));
                                a.name("Hatysa").url("https://sr.ht/~nerosnm/hatysa")
                            })
                            .field("Version", version, true)
                            .field(
                                "Uptime",
                                format!("{}d {}h {}m {}s", days, hours, minutes, seconds),
                                true,
                            )
                            .field("Homepage", homepage, false)
                            .colour((244, 234, 62));

                        m.set_embed(embed)
                    })
                    .await?;
            }
            Response::Pong => {
                debug!("ponging");
                self.message.channel_id.say(&self.ctx.http, "Pong!").await?;
            }
            Response::React { reactions } => {
                debug!("determining reaction target");

                // Find the message to react to.
                let target_id = self.find_previous_id().await?;

                debug!("getting target message by id");

                // Grab the actual message so we can add reactions.
                let target = self
                    .message
                    .channel_id
                    .message(&self.ctx.http, target_id)
                    .await
                    .map_err(|_| TaskError::GetMessage {
                        message_id: target_id,
                    })?;

                debug!("adding reactions");

                // React to the message.
                for reaction in reactions.into_iter().map(ReactionType::Unicode) {
                    target
                        .react(&self.ctx.http, reaction)
                        .await
                        .map_err(|_| TaskError::React {
                            message_id: target_id,
                        })?;
                }

                debug!("deleting original command message");

                // Delete the original message that triggered the task.
                self.message
                    .delete(&self.ctx.http)
                    .await
                    .map_err(|_| TaskError::Delete {
                        message_id: self.message.id,
                    })?;

                debug!("deleted original command message");
            }
            Response::Sketchify { url } => {
                debug!("building and sending a response containing the url");

                self.message
                    .channel_id
                    .say(
                        &self.ctx.http,
                        MessageBuilder::new()
                            .mention(&self.message.author.id)
                            .push(": <")
                            .push(url)
                            .push(">")
                            .build(),
                    )
                    .await?;

                debug!("deleting original command message");

                // Delete the original message that triggered the task.
                self.message
                    .delete(&self.ctx.http)
                    .await
                    .map_err(|_| TaskError::Delete {
                        message_id: self.message.id,
                    })?;

                debug!("deleted original command message");
            }
        }

        Ok(())
    }

    /// Find the ID of the message that occurred immediately before
    /// `self.message`.
    #[instrument(skip(self))]
    async fn find_previous_id(&self) -> Result<MessageId, TaskError> {
        debug!("searching for previous messages");

        let prev = self
            .message
            .channel_id
            .messages(&self.ctx.http, |retriever| {
                retriever.before(self.message.id).limit(1)
            })
            .await
            .map_err(|_| TaskError::GetPrevious {
                message_id: self.message.id,
            })?;

        debug!("getting single message from list of previous");

        let target = prev.first().ok_or(TaskError::GetPrevious {
            message_id: self.message.id,
        })?;

        debug!("found target message");

        Ok(target.id)
    }

    /// Attempt to report a command error to the user.
    #[instrument(skip(self), fields(channel_id = self.message.channel_id.0))]
    async fn report(&self, err: CommandError) -> Result<(), TaskError> {
        warn!("reporting error to user");

        let avatar_url = if let Ok(current_user) = self.ctx.http.get_current_user().await {
            current_user.avatar_url()
        } else {
            warn!("unable to retrieve avatar url for bot");
            None
        };

        match self
            .message
            .channel_id
            .send_message(&self.ctx.http, |m| {
                m.embed(|e| {
                    e.author(|a| {
                        if let Some(url) = avatar_url {
                            a.icon_url(url);
                        }

                        a.name("Hatysa").url("https://todo.sr.ht/~nerosnm/hatysa")
                    })
                    .field(
                        "Error",
                        match err {
                            CommandError::NonAlphanumeric { ref original } => MessageBuilder::new()
                                .push("String ")
                                .push_bold(original.to_uppercase())
                                .push(" contains non-alphanumeric characters!")
                                .build(),
                            CommandError::Repetition { ref original } => MessageBuilder::new()
                                .push("String ")
                                .push_bold(original.to_uppercase())
                                .push(" contains repeated characters!")
                                .build(),
                            CommandError::InvalidUrl(_) => {
                                MessageBuilder::new().push("Invalid URL!").build()
                            }
                            CommandError::Request(_) => MessageBuilder::new()
                                .push("Failed to complete request. Please try again.")
                                .build(),
                            CommandError::Internal(_) => MessageBuilder::new()
                                .push("An internal error occurred. Please try again later.")
                                .build(),
                        },
                        false,
                    )
                    .footer(|f| {
                        // TODO: Implement ,help command
                        // f.text(format!(
                        //     "For help, run {prefix}help. Click OK to delete.",
                        //     prefix = self.prefix
                        // ))
                        f.text("Click OK within 5 mins to delete.")
                    })
                    .colour((244, 234, 62))
                })
                .reactions(vec![ReactionType::Unicode("🆗".to_string())])
            })
            .await
        {
            Ok(sent_message) => {
                debug!("successfully reported error");

                if sent_message
                    .await_reaction(&self.ctx)
                    .filter(|react| react.emoji == ReactionType::Unicode("🆗".to_string()))
                    .author_id(self.message.author.id)
                    .timeout(Duration::from_secs(5 * 60))
                    .await
                    .is_some()
                {
                    debug!(
                        "got an OK reaction on error message {}, deleting",
                        sent_message.id
                    );

                    match sent_message.delete(&self.ctx.http).await {
                        Ok(_) => {
                            debug!("successfully deleted error message {}", sent_message.id);
                        }
                        Err(_) => {
                            error!("unable to delete error message {}", sent_message.id);
                        }
                    }

                    match self.message.delete(&self.ctx.http).await {
                        Ok(_) => {
                            debug!("successfully deleted original message {}", sent_message.id);
                            Ok(())
                        }
                        Err(_) => Err(TaskError::Delete {
                            message_id: self.message.id,
                        }),
                    }
                } else {
                    info!("no reaction received asking to delete message");

                    match sent_message
                        .delete_reaction_emoji(
                            &self.ctx.http,
                            ReactionType::Unicode("🆗".to_string()),
                        )
                        .await
                    {
                        Ok(_) => info!("deleted OK reaction prompt"),
                        Err(_) => warn!("unable to delete OK reaction prompt"),
                    }

                    Ok(())
                }
            }
            Err(_) => Err(TaskError::ReportError(err)),
        }
    }
}

/// Errors that could occur during task execution.
#[derive(thiserror::Error, Debug)]
pub enum TaskError {
    #[error("unable to send message: {0}")]
    SendMessage(#[from] serenity::Error),
    #[error("unable to react to message {}", message_id)]
    React { message_id: MessageId },
    #[error("unable to get message by id {}", message_id)]
    GetMessage { message_id: MessageId },
    #[error("unable to get message before message with id {}", message_id)]
    GetPrevious { message_id: MessageId },
    #[error("unable to delete message {}", message_id)]
    Delete { message_id: MessageId },
    #[error("unable to report command error: {0}")]
    ReportError(#[from] CommandError),
}

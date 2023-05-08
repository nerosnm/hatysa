//! Handle incoming events from Discord.
//!
//! The handler layer takes [`Context`][ctx] and [`Message`][msg] information as
//! input, and inspects them to determine if they should trigger a
//! [`Task`][task].
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
//! [ctx]: serenity::client::Context
//! [msg]: serenity::model::channel::Message
//! [task]: crate::task::Task
//! [execute]: hatysa::command::Command::execute

use chrono::{DateTime, Utc};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{channel::Message, gateway::Activity, gateway::Ready},
};
use tracing::{Instrument, Level};

use iota_orionis::command::Command;

use crate::task::Task;

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
    /// The date and time when this handler started running.
    pub start_time: DateTime<Utc>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        ctx.set_activity(Activity::playing(&*format!("{}react", self.prefix)))
            .await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let span = trace_span!("handler");
        async move {
            if let Some(command) = self.interpret_command(&msg).await {
                event!(
                    Level::DEBUG,
                    id = msg.id.0,
                    "message is a command, executing",
                );

                Task::new(command, ctx, msg).execute().await;
            } else {
                event!(Level::DEBUG, id = msg.id.0, "message is not a command");
            }
        }
        .instrument(span)
        .await;
    }
}

impl Handler {
    /// Attempt to parse a message as a command. If the message does not contain
    /// a valid command, `None` is returned.
    async fn interpret_command(&self, msg: &Message) -> Option<Command> {
        debug!("interpreting command");

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
            if let Some(tail) = tail.strip_prefix("clap").map(|tail| tail.trim()) {
                Some(Command::Clap {
                    input: tail.to_string(),
                })
            } else if tail.starts_with("info") {
                Some(Command::Info {
                    start_time: self.start_time,
                })
            } else if tail.starts_with("ping") {
                Some(Command::Ping)
            } else if let Some(tail) = tail.strip_prefix("react").map(|tail| tail.trim()) {
                Some(Command::React {
                    input: tail.to_owned(),
                })
            } else if let Some(tail) = tail.strip_prefix("sketchify").map(|tail| tail.trim()) {
                Some(Command::Sketchify {
                    url_raw: tail.to_owned(),
                })
            } else if let Some(tail) = tail.strip_prefix("spongebob").map(|tail| tail.trim()) {
                Some(Command::Spongebob {
                    input: tail.to_string(),
                })
            } else if let Some(tail) = tail.strip_prefix("wavy").map(|tail| tail.trim()) {
                Some(Command::Wavy {
                    input: tail.to_string(),
                })
            } else {
                tail.strip_prefix("zalgo")
                    .map(|tail| tail.trim())
                    .map(|tail| Command::Zalgo {
                        input: tail.to_string(),
                        max_chars: None,
                    })
            }
        } else {
            None
        }
    }
}

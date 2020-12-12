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
use lazy_static::lazy_static;
use regex::Regex;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{channel::Message, gateway::Activity, gateway::Ready},
};
use sqlx::sqlite::SqlitePool;
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
    /// A database connection pool.
    pub pool: SqlitePool,
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

        lazy_static! {
            static ref SIMPLE_INC: Regex = Regex::new(r"^(\w+)\+\+$").unwrap();
            static ref SIMPLE_DEC: Regex = Regex::new(r"^(\w+)\-\-$").unwrap();
        }

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
            } else if let Some(tail) = tail.strip_prefix("zalgo").map(|tail| tail.trim()) {
                Some(Command::Zalgo {
                    input: tail.to_string(),
                    max_chars: None,
                })
            } else if !msg.is_private() {
                // Commands that are not valid when run in a DM.
                if let Some(tail) = tail.strip_prefix("karma").map(|tail| tail.trim()) {
                    if tail.is_empty() {
                        Some(Command::KarmaTop {
                            pool: self.pool.clone(),
                        })
                    } else {
                        Some(Command::Karma {
                            subject: tail.to_string(),
                            pool: self.pool.clone(),
                        })
                    }
                } else {
                    None
                }
            } else {
                // The command wasn't recognised.
                None
            }
        } else {
            // The message didn't start with the prefix.
            if let Some(inc_captures) = SIMPLE_INC.captures(&msg.content) {
                Some(Command::KarmaIncrement {
                    subject: inc_captures
                        .get(1)
                        .expect("there should be a capture group 1 if the regex matched")
                        .as_str()
                        .to_string(),
                    pool: self.pool.clone(),
                })
            } else if let Some(dec_captures) = SIMPLE_DEC.captures(&msg.content) {
                Some(Command::KarmaDecrement {
                    subject: dec_captures
                        .get(1)
                        .expect("there should be a capture group 1 if the regex matched")
                        .as_str()
                        .to_string(),
                    pool: self.pool.clone(),
                })
            } else {
                None
            }
        }
    }
}

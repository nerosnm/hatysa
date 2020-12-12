//! Execute commands and return their output.

mod clap;
mod info;
mod react;
mod sketchify;
mod spongebob;
mod wavy;
mod zalgo;

#[cfg(feature = "persistence")]
mod karma;

use chrono::{DateTime, Utc};
use url::{ParseError, Url};

#[cfg(feature = "persistence")]
use sqlx::sqlite::SqlitePool;

/// Commands that can be performed.
#[derive(Debug)]
pub enum Command {
    /// Insert clapping emojis between every word of the input text.
    Clap {
        /// The input to convert.
        input: String,
    },
    /// A request from a user for some information about the currently running
    /// instance of the bot.
    Info {
        /// The start time of this bot instance.
        start_time: DateTime<Utc>,
    },
    /// Get the current karma of a subject.
    #[cfg(feature = "persistence")]
    Karma {
        /// The subject.
        subject: String,
        /// A pool of connections to a database where the karma is stored.
        pool: SqlitePool,
    },
    /// Get a list of the subjects with the most karma.
    #[cfg(feature = "persistence")]
    KarmaTop {
        /// A pool of connections to a database where the karma is stored.
        pool: SqlitePool,
    },
    /// Decrease the karma of a subject.
    #[cfg(feature = "persistence")]
    KarmaDecrement {
        /// The subject.
        subject: String,
        /// A pool of connections to a database where the karma is stored.
        pool: SqlitePool,
    },
    /// Increase the karma of a subject.
    #[cfg(feature = "persistence")]
    KarmaIncrement {
        /// The subject.
        subject: String,
        /// A pool of connections to a database where the karma is stored.
        pool: SqlitePool,
    },
    /// A request from a user for a response, to check if the bot is alive.
    Ping,
    /// Convert an input string into a series of emojis that can then be used to
    /// react to a message.
    React {
        /// The string to convert to emojis.
        input: String,
    },
    /// Convert a URL to a "sketchified" equivalent using [the Sketchify
    /// API][sketchify].
    ///
    /// [sketchify]: https://verylegit.link
    Sketchify {
        /// The string provided for the URL to sketchify.
        url_raw: String,
    },
    /// Convert text to Spongebob-case text.
    Spongebob {
        /// The input to convert.
        input: String,
    },
    /// Convert text to vaporwave (fullwidth) text.
    Wavy {
        /// The input to convert.
        input: String,
    },
    /// Convert text to Zalgo text.
    Zalgo {
        /// The input to convert.
        input: String,
        /// If provided, the maximum number of characters to output.
        max_chars: Option<usize>,
    },
}

impl Command {
    /// Execute a command, returning its response.
    pub async fn execute(self) -> Result<Response, CommandError> {
        match self {
            Command::Clap { input } => Ok(clap::clap(input)),
            Command::Info { start_time } => Ok(info::info(start_time).await),
            #[cfg(feature = "persistence")]
            Command::Karma { subject, pool } => karma::get(subject, pool).await,
            #[cfg(feature = "persistence")]
            Command::KarmaTop { pool } => karma::top(pool).await,
            #[cfg(feature = "persistence")]
            Command::KarmaDecrement { subject, pool } => karma::dec(subject, pool).await,
            #[cfg(feature = "persistence")]
            Command::KarmaIncrement { subject, pool } => karma::inc(subject, pool).await,
            Command::Ping => Ok(Response::Pong),
            Command::React { input } => react::react(input),
            Command::Sketchify { url_raw } => sketchify::sketchify(url_raw).await,
            Command::Spongebob { input } => Ok(spongebob::spongebob(input)),
            Command::Wavy { input } => wavy::wavy(input),
            Command::Zalgo { input, max_chars } => Ok(zalgo::zalgo(input, max_chars)),
        }
    }
}

/// Possible responses as a result of a command.
#[derive(Debug)]
pub enum Response {
    /// Response to a [Command::Clap].
    Clap {
        /// The converted input.
        output: String,
    },
    /// Response to a [Command::Info].
    Info {
        /// The current version of the bot.
        version: String,
        /// Uptime, in the form `(days, hours, minutes, seconds)`.
        uptime: (i64, i64, i64, i64),
        /// The homepage of the bot.
        homepage: String,
    },
    /// Response to a [Command::Karma].
    #[cfg(feature = "persistence")]
    Karma {
        /// The subject.
        subject: String,
        /// The amount of karma the subject has.
        karma: u32,
    },
    /// Response to a [Command::KarmaTop].
    #[cfg(feature = "persistence")]
    KarmaTop {
        /// The top subjects, sorted by their karma.
        top: Vec<karma::Karma>,
        /// The amount of karma the subject has.
        karma: u32,
    },
    /// Response to a [Command::KarmaDecrement].
    #[cfg(feature = "persistence")]
    KarmaDecrement,
    /// Response to a [Command::KarmaIncrement].
    #[cfg(feature = "persistence")]
    KarmaIncrement,
    /// Response to a [Command::Ping].
    Pong,
    /// Response to a [Command::React].
    React {
        /// A sequence of emojis created to represent the input string.
        reactions: Vec<String>,
    },
    /// Response to a [Command::Sketchify].
    Sketchify {
        /// The converted URL.
        url: Url,
    },
    /// Response to a [Command::Spongebob].
    Spongebob {
        /// The converted input.
        output: String,
    },
    /// Response to a [Command::Wavy].
    Wavy {
        /// The converted input.
        output: String,
    },
    /// Response to a [Command::Zalgo].
    Zalgo {
        /// The converted input.
        output: String,
    },
}

/// Errors that could occur during command processing.
#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("string \"{}\" contains non-alphanumeric characters", original)]
    NonAlphanumeric { original: String },
    #[error("string \"{}\" contains repeated characters", original)]
    Repetition { original: String },
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] ParseError),
    #[error("could not complete request: {0}")]
    Request(#[from] reqwest::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

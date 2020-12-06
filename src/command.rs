//! Execute commands and return their output.

mod clap;
mod info;
mod react;
mod sketchify;
mod spongebob;
mod vape;
mod zalgo;

use chrono::{DateTime, Utc};
use url::{ParseError, Url};

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
    Vape {
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
            Command::Ping => Ok(Response::Pong),
            Command::React { input } => react::react(input),
            Command::Sketchify { url_raw } => sketchify::sketchify(url_raw),
            Command::Spongebob { input } => Ok(spongebob::spongebob(input)),
            Command::Vape { input } => vape::vape(input),
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
    /// Response to a [Command::Vape].
    Vape {
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

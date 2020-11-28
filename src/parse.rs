//! Parse arguments to commands.

use structopt::{clap::AppSettings, StructOpt};
use url::Url;

#[derive(StructOpt, Debug, Clone, PartialEq, Eq)]
#[structopt(
    global_settings = &[
        AppSettings::NoBinaryName,
        AppSettings::DisableHelpFlags,
        AppSettings::DisableVersion,
    ],
)]
pub enum Options {
    /// Put 👏 clap 👏 emojis 👏 after 👏 each 👏 word.
    Clap {
        /// The text to emphasise.
        input: String,
    },
    /// Request info about the currently running bot instance.
    Info,
    /// Ping the bot, to check if it's alive.
    Ping,
    /// React to the previous message with emojis spelling out a word.
    React {
        /// The word (unique alphanumeric characters only) to react with.
        reaction: String,
    },
    /// Turn a link into a much sketchier looking version using
    /// https://verylegitlink.
    Sketchify {
        /// The URL to sketchify.
        url: Url,
    },
    /// Convert text to Spongebob-case text.
    Spongebob {
        /// The text to convert.
        input: String,
    },
    /// Convert text to vaporwave text.
    Vape {
        /// The text to convert.
        input: String,
    },
    /// H̛̹͝e̳̼͙ ̤̎͝c͓̺̎ȏ͇ͤm̨͡͠e͚ͫ͡s͗ͭ͢
    Zalgo {
        /// The text to turn into Zalgo text.
        #[structopt(required = true, min_values = 1)]
        input: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use structopt::StructOpt;

    use super::*;

    #[test]
    fn parse_ping() {
        let input = "ping".to_string();
        let input_iter = input.split_whitespace();

        let actual = Options::from_iter_safe(input_iter)
            .expect(&*format!("input {:?} should successfully parse", input));
        let expected = Options::Ping;

        assert_eq!(
            actual, expected,
            "the input {:?} did not parse as {:#?}",
            input, expected
        );
    }

    #[test]
    fn parse_react() {
        let input = "react o0f69".to_string();
        let input_iter = input.split_whitespace();

        let actual = Options::from_iter_safe(input_iter)
            .expect(&*format!("input {:?} should successfully parse", input));
        let expected = Options::React {
            reaction: "o0f69".to_string(),
        };

        assert_eq!(
            actual, expected,
            "the input {:?} did not parse as {:#?}",
            input, expected
        );
    }

    #[test]
    fn parse_react_with_spaces() {
        let input = "react o0f69 asdfjasdf".to_string();
        let input_iter = input.split_whitespace();

        let _ = Options::from_iter_safe(input_iter)
            .expect_err(&*format!("input {:?} should not successfully parse", input));
    }

    #[test]
    fn parse_react_no_reaction() {
        let input = "react".to_string();
        let input_iter = input.split_whitespace();

        let _ = Options::from_iter_safe(input_iter)
            .expect_err(&*format!("input {:?} should not successfully parse", input));
    }

    #[test]
    fn parse_sketchify() {
        let input = "sketchify https://git.sr.ht".to_string();
        let input_iter = input.split_whitespace();

        let actual = Options::from_iter_safe(input_iter)
            .expect(&*format!("input {:?} should successfully parse", input));
        let expected = Options::Sketchify {
            url: Url::parse("https://git.sr.ht").unwrap(),
        };

        assert_eq!(
            actual, expected,
            "the input {:?} did not parse as {:#?}",
            input, expected
        );
    }

    #[test]
    fn parse_sketchify_no_url() {
        let input = "sketchify".to_string();
        let input_iter = input.split_whitespace();

        let _ = Options::from_iter_safe(input_iter)
            .expect_err(&*format!("input {:?} should not successfully parse", input));
    }

    #[test]
    fn parse_sketchify_invalid_url() {
        let input = "sketchify %393j+}[4".to_string();
        let input_iter = input.split_whitespace();

        let _ = Options::from_iter_safe(input_iter)
            .expect_err(&*format!("input {:?} should not successfully parse", input));
    }

    #[test]
    fn parse_sketchify_with_spaces() {
        let input = "sketchify https://git.sr.ht https://lobste.rs".to_string();
        let input_iter = input.split_whitespace();

        let _ = Options::from_iter_safe(input_iter)
            .expect_err(&*format!("input {:?} should not successfully parse", input));
    }

    #[test]
    fn parse_zalgo() {
        let input = "zalgo ZALGO!".to_string();
        let input_iter = input.split_whitespace();

        let actual = Options::from_iter_safe(input_iter)
            .expect(&*format!("input {:?} should successfully parse", input));
        let expected = Options::Zalgo {
            input: vec!["ZALGO!".to_string()],
        };

        assert_eq!(
            actual, expected,
            "the input {:?} did not parse as {:#?}",
            input, expected
        );
    }

    #[test]
    fn parse_zalgo_no_input() {
        let input = "zalgo".to_string();
        let input_iter = input.split_whitespace();

        let _ = Options::from_iter_safe(input_iter)
            .expect_err(&*format!("input {:?} should not successfully parse", input));
    }

    #[test]
    fn parse_zalgo_with_spaces() {
        let input = "zalgo He who Waits Behind The Wall".to_string();
        let input_iter = input.split_whitespace();

        let actual = Options::from_iter_safe(input_iter)
            .expect(&*format!("input {:?} should successfully parse", input));

        let expected = Options::Zalgo {
            input: "He who Waits Behind The Wall"
                .split_whitespace()
                .map(ToOwned::to_owned)
                .collect(),
        };

        assert_eq!(
            actual, expected,
            "the input {:?} did not parse as {:#?}",
            input, expected
        );
    }
}

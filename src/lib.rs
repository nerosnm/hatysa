//! Hatysa is a Discord bot that implements a few fun commands.
//!
//! ## Usage
//!
//! ### Discord Bot
//!
//! Install the bot from [crates.io](https://crates.io):
//!
//! ```bash
//! $ cargo install hatysa
//! ```
//!
//! To run the bot, you'll need to provide a Discord token (obtainable from the
//! [Discord Developer Portal](https://discord.com/developers)), as follows:
//!
//! ```bash
//! $ DISCORD_TOKEN="<token>" hatysa
//! ```
//!
//! The prefix can be changed from the default (`,`) using `HATYSA_PREFIX`, and
//! you might also want to [change the tracing subscriber
//! filter][tracing-subscriber] to customise what log messages are
//! printed out:
//!
//! [tracing-subscriber]:
//! ../tracing_subscriber/fmt/index.html#filtering-events-with-environment-variables
//!
//! ```bash
//! $ DISCORD_TOKEN="<token>" HATYSA_PREFIX="!" RUST_LOG="info,hatysa=debug" hatysa
//! ```
//!
//! ### Backend
//!
//! The backend of the bot is available as a library, to make use of any of its
//! commands without interacting with Discord. To use the crate, just add the
//! following to your `Cargo.toml` file, where `<version>` is the version
//! obtained from `cargo search hatysa`:
//!
//! ```toml
//! [dependencies.hatysa]
//! version = "<version>"
//! default-features = false
//! ```
//!
//! > `default-features = false` disables the dependency on the `serenity`
//! Discord client library, since it's only used by the binary target that
//! implements the Discord bot.
//!
//! ## License
//!
//! Licensed under either of
//!
//! - Apache License, Version 2.0 (LICENSE-APACHE or
//! <http://www.apache.org/licenses/LICENSE-2.0>)
//! - MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
//!
//! ### Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally
//! submitted for inclusion in the work by you, as defined in the Apache-2.0
//! license, shall be dual licensed as above, without any additional terms or
//! conditions.
//!
//! ## Resources
//!
//! [Send patches](https://git-send-email.io) and questions to
//! [~nerosnm/hatysa-devel@lists.sr.ht](https://lists.sr.ht/~nerosnm/hatysa-devel).
//!
//! Bug & issue tracker: [~nerosnm/hatysa](https://todo.sr.ht/~nerosnm/hatysa).

pub mod command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
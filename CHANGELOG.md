# Changelog

## 0.3.0

### Features

- Comprehensive instrumentation using `tracing` (#2).

### Miscellaneous

- Renamed `,vape` command to `,wavy`.
- Made command processing independent of any Discord context (#9).
- Split crate into a library and a binary, so that hatysa can be used from other Rust crates.

## 0.2.1

### Features

- `,vape` command to convert all the characters in the input into their Unicode fullwidth 
equivalents (#6).
- `,info` command to return some info about the running instance, including version and uptime.
- Make prefix optional when sending commands in DMs.
- Report errors to user with embeds rather than individual messages (#5).

## 0.2.0

### Features

- `,clap` command to insert clapping emojis between each word of the input text (#4).
- `,spongebob` command to convert input text into alternating upper- and lower-case characters.
- `,zalgo` command to convert the input into Zalgo text.

### Bug Fixes

- `,react` rejects input strings that contain spaces (#1).

## 0.1.0

### Features

- `,react` command to convert a string to a series of emojis and react to the last message in the 
channel with those emojis.
- `,sketchify` command to convert a URL to a sketchy-looking equivalent using the converter at 
https://verylegit.link.
- `,ping` command to check that the bot is alive.


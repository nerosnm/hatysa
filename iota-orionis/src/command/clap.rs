//! Insert clapping emojis between every word of the input text.

use super::Response;

#[instrument]
pub fn clap(input: String) -> Response {
    let mut words = input.split(' ');

    let clappified = words
        .next()
        .map(|first| first.to_string())
        .map(|first| {
            words.fold(first, |mut acc, next| {
                acc.push_str(&format!(" 👏 {}", next));
                acc
            })
        })
        .map(|mut output| {
            output.push_str(" 👏");
            output
        })
        .unwrap_or_else(String::new);

    let response = Response::Clap { output: clappified };

    debug!(?response);

    response
}

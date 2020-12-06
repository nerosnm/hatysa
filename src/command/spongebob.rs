//! Convert text to Spongebob-case text.

use tracing::{debug, instrument};

use super::Response;

#[instrument]
pub fn spongebob(input: String) -> Response {
    let (_, spongebobified) =
        input
            .chars()
            .fold((false, String::new()), |(upper, mut output), next_char| {
                let next_upper = if next_char.is_alphanumeric() {
                    if upper {
                        output.push(next_char.to_ascii_uppercase());
                    } else {
                        output.push(next_char.to_ascii_lowercase());
                    }

                    !upper
                } else {
                    output.push(next_char);
                    upper
                };

                (next_upper, output)
            });

    let response = Response::Spongebob {
        output: spongebobified,
    };

    debug!(?response);

    response
}

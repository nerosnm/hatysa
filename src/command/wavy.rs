//! Convert text to vaporwave (fullwidth) text.


use super::{CommandError, Response};

#[instrument]
pub fn wavy(input: String) -> Result<Response, CommandError> {
    debug!(?input);

    let response = Response::Wavy {
        output: wavify(input)?,
    };

    debug!(?response);

    Ok(response)
}

fn wavify(input: String) -> Result<String, CommandError> {
    input
        .chars()
        .map(|c| {
            let val = c as u32;
            match val {
                0x0020 => std::char::from_u32(0x3000).ok_or(CommandError::Internal(
                    "Invalid fullwidth space character".to_string(),
                )),
                0x0021..=0x007e => std::char::from_u32(val + 0xfee0).ok_or(CommandError::Internal(
                    "fullwidth character equivalent should be valid".to_string(),
                )),
                _ => Ok(c),
            }
        })
        .collect::<Result<String, CommandError>>()
}

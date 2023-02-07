//! Convert text to Zalgo text.

use rand::Rng;

use super::Response;

#[instrument]
pub fn zalgo(input: String, max_chars: Option<usize>) -> Response {
    let per_char = max_chars
        .map(|max_chars| (max_chars - input.len()) / input.len())
        .map(|per_char| per_char.min(10))
        .unwrap_or_else(|| 10);

    let response = Response::Zalgo {
        output: zalgify(input, per_char),
    };

    debug!(?response);

    response
}

fn zalgify(input: String, per_char: usize) -> String {
    input
        .chars()
        .flat_map(|c| {
            let mut rng = rand::thread_rng();
            let combiners = (0..per_char).map(|_| {
                let val = rng.gen_range(0x300..0x36f);
                unsafe { std::char::from_u32_unchecked(val) }
            });

            std::iter::once(c).chain(combiners).collect::<Vec<char>>()
        })
        .collect::<String>()
}

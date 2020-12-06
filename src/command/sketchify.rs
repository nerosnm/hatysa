//! The sketchify command converts a URL into a "sketchy" version using [the
//! Sketchify API][sketchify].
//!
//! [sketchify]: https://verylegit.link

use url::Url;

use super::{CommandError, Response};

#[instrument]
pub async fn sketchify(url_raw: String) -> Result<Response, CommandError> {
    debug!(?url_raw);

    let url = Url::parse(&*url_raw)
        .or_else(|_| Url::parse(&*format!("http://{}", url_raw)))
        .map_err(|err| {
            warn!("failed to parse URL");
            err
        })?;

    let api_params = [("long_url", url.to_string())];
    debug!(?api_params);

    let client = reqwest::Client::new();
    let res = client
        .post("http://verylegit.link/sketchify")
        .form(&api_params)
        .send()
        .await
        .map_err(|err| {
            error!("failed to send request");
            err
        })?;
    debug!(?res);

    let sketchified_url_str = res.text().await.map_err(|err| {
        error!("failed to extract text from API response");
        err
    })?;

    let sketchified_url = if !sketchified_url_str.starts_with("http") {
        Url::parse(&*format!("http://{}", sketchified_url_str))
    } else {
        Url::parse(&*sketchified_url_str)
    }
    .map_err(|err| {
        error!("failed to parse returned URL");
        err
    })?;

    let response = Response::Sketchify {
        url: sketchified_url,
    };

    debug!(?response);

    Ok(response)
}

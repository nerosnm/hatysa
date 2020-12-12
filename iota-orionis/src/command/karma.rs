//! Track the karma of subjects.

#![allow(unused_variables)]

use sqlx::sqlite::SqlitePool;

use super::{CommandError, Response};

#[derive(Debug)]
pub struct Karma {
    subject: String,
    karma: u32,
}

#[instrument(skip(pool))]
pub async fn get(subject: String, pool: SqlitePool) -> Result<Response, CommandError> {
    debug!("getting karma");

    Ok(Response::Karma { subject, karma: 0 })
}

#[instrument(skip(pool))]
pub async fn top(pool: SqlitePool) -> Result<Response, CommandError> {
    debug!("getting top karma");

    let top = vec![];

    Ok(Response::KarmaTop { top, karma: 0 })
}

#[instrument(skip(pool))]
pub async fn inc(subject: String, pool: SqlitePool) -> Result<Response, CommandError> {
    info!("incrementing karma");

    Ok(Response::KarmaIncrement)
}

#[instrument(skip(pool))]
pub async fn dec(subject: String, pool: SqlitePool) -> Result<Response, CommandError> {
    info!("decrementing karma");

    Ok(Response::KarmaDecrement)
}

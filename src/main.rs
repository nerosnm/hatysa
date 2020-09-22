fn main() -> eyre::Result<()> {
    let params = [("long_url", "git.sr.ht/~nerosnm/sketchify-bot")];
    let client = reqwest::Client::new();
    let mut res = client
        .post("http://verylegit.link/sketchify")
        .form(&params)
        .send()?;

    println!("{}", res.text()?);

    Ok(())
}

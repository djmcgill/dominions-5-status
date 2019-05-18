use reqwest;
use std::error::Error;
use url::Url;

pub fn snek_details(address: &str) -> Result<Option<()>, Box<Error>> {
    let snek_url = Url::parse(address)?;

    let host_str = snek_url.host_str().ok_or_else(|| -> Box<Error> {
        format!("Url '{}' did not have host", address).into()
    })?;
    if host_str != "snek.earth" && host_str != "dom5.snek.earth" {
        return Ok(None);
    }
    let port = snek_url.port().ok_or_else(|| -> Box<Error> {
        format!("Url '{}' did not have port", address).into()
    })?;

    if port <= 30_000 {
        return Err("Url '{}' had an invalid port".into())
    };
    let game_id = port-30_000;

    let response = reqwest::get(&format!("https://dom5.snek.earth/api/games/{}/status", game_id))?;

    Ok(Some(()))
}

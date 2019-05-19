use reqwest;
use reqwest::StatusCode;
use std::error::Error;
use std::str::FromStr;
use url::Url;

use serde::de;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

pub struct SnekGameStatus {
    pub nations: HashMap<u32, SnekNation>,
}

#[derive(Deserialize, Debug)]
struct RawSnekGameStatus {
    nations: Vec<SnekNation>,
}

#[derive(Deserialize, Debug)]
pub struct SnekNation {
    #[serde(rename = "nationid", deserialize_with = "u32_from_str")]
    pub nation_id: u32,
    pub name: String,
}

fn u32_from_str<'de, D>(d: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    u32::from_str(&s).map_err(de::Error::custom)
}

pub fn snek_details(address: &str) -> Result<Option<SnekGameStatus>, Box<Error>> {
    let snek_url = Url::parse(&format!("https://{}", address)).or_else(|_| Url::parse(address))?;

    println!("SNEK URL: '{:?}'", snek_url.host_str());
    let host_str = snek_url
        .host_str()
        .ok_or_else(|| -> Box<Error> { format!("Url '{}' did not have host", address).into() })?;
    if host_str != "snek.earth" && host_str != "dom5.snek.earth" {
        return Ok(None);
    }
    let port = snek_url
        .port()
        .ok_or_else(|| -> Box<Error> { format!("Url '{}' did not have port", address).into() })?;

    if port <= 30_000 {
        return Err("Url '{}' had an invalid port".into());
    };
    let game_id = port - 30_000;

    let mut response = reqwest::get(&format!(
        "https://dom5.snek.earth/api/games/{}/status",
        game_id
    ))?;
    if response.status() != StatusCode::OK {
        return Err("Snek did not respond with OK".into());
    }
    let parsed_response = response.json::<RawSnekGameStatus>()?;

    let mut hash_map = HashMap::new();
    for nation in parsed_response.nations {
        hash_map.insert(nation.nation_id, nation);
    }

    Ok(Some(SnekGameStatus { nations: hash_map }))
}

use anyhow::Context;
use reqwest::StatusCode;
use serde::{de, Deserialize, Deserializer};
use std::{collections::HashMap, str::FromStr, time::Duration};
use tokio::time;
use url::Url;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SnekGameStatus {
    pub nations: HashMap<u32, SnekNation>,
}

#[derive(Deserialize, Debug)]
struct RawSnekGameStatus {
    nations: Vec<SnekNation>,
}

#[derive(PartialEq, Eq, Deserialize, Debug, Clone)]
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

pub async fn snek_details_async(address: &str) -> anyhow::Result<Option<SnekGameStatus>> {
    let snek_url = Url::parse(&format!("https://{}", address)).or_else(|_| Url::parse(address))?;

    let host_str = snek_url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Url '{}' did not have host", address))?;

    if host_str != "snek.earth" && host_str != "dom5.snek.earth" {
        return Ok(None);
    }
    let port = snek_url
        .port()
        .ok_or_else(|| anyhow::anyhow!("Url '{}' did not have port", address))?;

    if port <= 30_000 {
        return Err(anyhow::anyhow!("Url '{}' had an invalid port", address));
    };
    let game_id = port - 30_000;

    let response = time::timeout(
        Duration::from_secs(5),
        reqwest::get(&format!(
            "https://dom5.snek.earth/api/games/{}/status",
            game_id
        )),
    )
    .await
    .context("timed out getting snek info")?
    .context("failed to get snek info")?;
    if response.status() != StatusCode::OK {
        return Err(anyhow::anyhow!("Snek did not respond with OK"));
    }
    let parsed_response = response.json::<RawSnekGameStatus>().await?;

    let mut hash_map = HashMap::new();
    for nation in parsed_response.nations {
        hash_map.insert(nation.nation_id, nation);
    }

    Ok(Some(SnekGameStatus { nations: hash_map }))
}

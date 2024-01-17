use crate::model::{
    enums::{NationStatus, SubmissionStatus},
    game_data::GameData,
    nation::{GameNationIdentifier, Nation},
    raw_game_data::RawGameData,
};
use anyhow::{anyhow, Context};
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::Utc;
use flate2::read::ZlibDecoder;
use log::*;
use scraper::{Html, Selector};
use std::{
    io::{BufRead, Cursor, Read},
    str::FromStr,
    time::Duration,
};
use tokio::{io::AsyncWriteExt, time};

pub async fn get_game_data_async(
    server_address: &str,
    dom_version: u8,
) -> anyhow::Result<GameData> {
    let option_url = url::Url::parse(server_address).ok();
    let is_html_page = option_url
        .as_ref()
        .and_then(|url| url.path().split('.').last().map(|s| s.to_owned()))
        .is_some_and(|x| x == "html");
    if is_html_page {
        // assume html pages are dom 6 only
        let response = time::timeout(
            Duration::from_secs(5),
            reqwest::get(option_url.expect("we already checked this")),
        )
        .await
        .context("retrieving html page from the server timed out")?
        .context("cannot get html page from the server")?;
        let text = response
            .text()
            .await
            .context("failed to decode html response body")?;
        parse_status_html(Html::parse_document(&text))
    } else {
        // assume it's direct connect
        let raw_data = time::timeout(
            Duration::from_secs(5),
            get_raw_game_data_async(server_address),
        )
        .await
        .context("retrieving info from the server timed out")?
        .context("cannot retrieve info from the server")?;
        let game_data = interpret_raw_data(raw_data, dom_version)?;
        Ok(game_data)
    }
}

fn parse_status_html(page: Html) -> anyhow::Result<GameData> {
    let table_selector = Selector::parse("table").unwrap();
    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let table = page
        .select(&table_selector)
        .next()
        .ok_or_else(|| anyhow!("No <table> found"))?;
    let mut rows = table.select(&tr_selector);
    let header_row = rows.next().ok_or_else(|| anyhow!("No header <tr> found"))?;
    let header_element = header_row
        .select(&td_selector)
        .next()
        .ok_or_else(|| anyhow!("No header <td> found"))?
        .inner_html();

    // samog, turn 41 (finished)
    // samog, turn 41 (time left: 1 days and 1 hours)
    let mut header_sections = header_element.split(',');
    let game_name = header_sections
        .next()
        .ok_or_else(|| anyhow!("No game_name section in the header found"))?
        .to_owned();
    let remaining_header = header_sections
        .next()
        .ok_or_else(|| anyhow!("No remaining_header found"))?;
    // skip " turn "
    let mut remaining_header = remaining_header.chars().skip(6);
    let digits = (&mut remaining_header)
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();
    let turn = i32::from_str(&digits).context("parse turn")?;

    // skip " (time left: "
    let remaining_header = remaining_header.skip(13).collect::<String>();

    // 1 days and 1 hours
    // okay I'm a bit worried we might see "1 days" or "1 day" or "1 hour" or "1 hours"
    // depending on if there's e.g. exactly 24 hours or <24 hours remaining
    let mut words = remaining_header.split(' ');
    let (finished, turn_deadline) = if let Some(first_count) = words.next() {
        let mut seconds = 0;

        if !first_count.is_empty() {
            let first_unit = words.next().ok_or_else(|| anyhow!("first_unit"))?;
            let first_count = i32::from_str(first_count).context("first_count i32 parse")?;
            match first_unit {
                "week" | "weeks" => seconds += first_count * 7 * 24 * 60 * 60,
                "days" | "day" => seconds += first_count * 24 * 60 * 60,
                "hours" | "hour" => seconds += first_count * 60 * 60,
                "minute" | "minutes" => seconds += first_count * 60,
                "second" | "seconds" => seconds += first_count,
                _ => return Err(anyhow!("Unknown first_unit: '{}'", first_unit)),
            }
            // skip 'and'
            let _ = words.next();
            let second_unit = words.next();
            let second_count = words.next();
            match (second_unit, second_count) {
                (Some(second_unit), Some(second_count)) => {
                    let second_count =
                        i32::from_str(second_count).context("second_count i32 parse");
                    match second_unit {
                        "days)" | "day)" => seconds += second_count? * 24 * 60 * 60,
                        "hours)" | "hour)" => seconds += second_count? * 60 * 60,
                        "minute)" | "minutes)" => seconds += second_count? * 60,
                        "second)" | "seconds)" => seconds += second_count?,
                        _ => return Err(anyhow!("Unknown first_unit: '{}'", first_unit)),
                    }
                }
                _ => {
                    return Err(anyhow!(
                        "Okay I really don't know what's going on at this point"
                    ))
                }
            }
            if let Some(remaining) = words.next() {
                return Err(anyhow!("Extra time text remaining: '{}'", remaining));
            }
        }
        // yes this does not cover leap seconds or
        // daylight savings properly however: I do not give a shit
        (false, Utc::now() + Duration::from_secs(seconds as u64))
    } else {
        // finished
        (true, Utc::now())
    };

    let nations = rows
        .map(|row| {
            let mut cells = row.select(&td_selector);
            let name = cells
                .next()
                .ok_or_else(|| anyhow!("No name <td> found for row"))?
                .inner_html();
            let identifier = GameNationIdentifier::from_name_6(&name)
                .with_context(|| format!("parse nation name: '{}'", &name))?;

            let status = cells
                .next()
                .ok_or_else(|| anyhow!("No status <td> found for row"))?
                .inner_html();
            let (submitted, status) = if finished {
                (SubmissionStatus::Submitted, NationStatus::DefeatedThisTurn)
            } else {
                match status.as_ref() {
                    "Turn played" => (SubmissionStatus::Submitted, NationStatus::Human),
                    "Turn unfinished" => {
                        (SubmissionStatus::PartiallySubmitted, NationStatus::Human)
                    }
                    "-" => (SubmissionStatus::NotSubmitted, NationStatus::Human),
                    "Eliminated" => (SubmissionStatus::Submitted, NationStatus::Defeated),
                    _ => return Err(anyhow!("Unknown player status: '{}'", status)),
                }
            };
            Ok(Nation {
                identifier,
                submitted,
                status,
                connected: false,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(GameData {
        game_name,
        nations,
        turn,
        turn_deadline,
    })
}

fn interpret_raw_data(raw_data: RawGameData, dom_version: u8) -> anyhow::Result<GameData> {
    let turn_deadline = Utc::now()
        .checked_add_signed(chrono::Duration::milliseconds(raw_data.d.into()))
        .ok_or_else(|| anyhow!("invalid duration remaining in turn"))?;

    let mut game_data = GameData {
        game_name: raw_data.game_name,
        nations: vec![],
        turn: raw_data.h as i32,
        turn_deadline,
    };
    for i in 0..250 {
        let status_num = raw_data.f[i];
        if status_num != 0 && status_num != 3 {
            let submitted = raw_data.f[i + 250];
            let connected = raw_data.f[i + 500];
            let nation_id = (i - 1) as u32; // why -1? No fucking idea
            let identifier = match dom_version {
                5 => GameNationIdentifier::from_id(nation_id),
                6 => GameNationIdentifier::from_id_6(nation_id),
                _ => return Err(anyhow!("Dom {} lol", dom_version)),
            };
            let nation = Nation {
                identifier,
                status: NationStatus::from_int(status_num)
                    .ok_or_else(|| anyhow!("Unknown nation status {}", status_num))?,
                submitted: SubmissionStatus::from_int(submitted),
                connected: connected == 1,
            };
            game_data.nations.push(nation);
        }
    }
    Ok(game_data)
}
async fn get_raw_game_data_async(server_address: &str) -> anyhow::Result<RawGameData> {
    let buffer = call_server_for_info_async(server_address).await?;
    let decompressed = decompress_server_info(&buffer)?;
    let game_data = parse_data(&decompressed)?;
    Ok(game_data)
}
async fn call_server_for_info_async(server_address: &str) -> anyhow::Result<Vec<u8>> {
    use tokio::io::AsyncReadExt;
    debug!("call_server_for_info_async for {}", server_address);
    let mut stream = tokio::net::TcpStream::connect(server_address).await?;

    // No idea where this means lol
    // It's a modification of the original dom3/4 script, it got changed in patch 5.44
    // https://steamcommunity.com/app/722060/discussions/0/1749024748627269322/?ctp=2#c1749024925634051868
    let request = [
        b'f', b'H', // wtr.write_u32::<LittleEndian>(1)?;
        // wtr.write_u8(3)?;
        0x07, 0x00, 0x00, 0x00, b'=', 0x1e, 0x02, 0x11, b'E', 0x05, 0x00,
    ];

    stream.write_all(&request).await?;

    let mut header_buffer = [0; 6];
    debug!("trying to receive");
    let header_bytes_read = stream.read_exact(&mut header_buffer).await?;
    debug!(
        "received {} header bytes, byte 2 is {}",
        header_bytes_read, header_buffer[2],
    );

    let mut body_buffer = vec![0; header_buffer[2] as usize];
    let body_bytes_read = stream.read_exact(&mut body_buffer).await?;
    debug!("received {} body bytes", body_bytes_read);

    debug!("sending close");
    let close_request = [
        b'f', b'H', 0x01, 0x00, 0x00, 0x00, // ::<LittleEndian>(1)?;
        11,
    ];
    stream.write_all(&close_request).await?;

    let mut buffer = header_buffer.to_vec();
    buffer.append(&mut body_buffer);

    Ok(buffer)
}
fn decompress_server_info(raw: &[u8]) -> anyhow::Result<Vec<u8>> {
    debug!("HEADER {:?}", &raw[0..10]);
    if raw[1] == b'J' {
        debug!("decompressing");
        let mut decoder = ZlibDecoder::new(&raw[10..]);
        let mut decompressed = vec![];
        let _ = decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    } else {
        debug!("No need to decompress");
        Ok(raw[10..].to_vec())
    }
}
fn parse_data(data: &[u8]) -> anyhow::Result<RawGameData> {
    let len = data.len();
    debug!("done: data.len(): {}", len);

    let mut cursor = Cursor::new(data);
    let mut a = [0u8; 6];
    Read::read_exact(&mut cursor, &mut a)?;
    debug!(
        "cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    // debug!("A: {:#?}", a);
    // debug!("Au32b: {}", u32::from_be_bytes([a[0], a[1]]));
    // debug!("Au32l: {}", u32::from_le_bytes([a[0], a[1]]));
    debug!("parsing name");
    let mut game_name_bytes = vec![];
    let read_bytes = cursor.read_until(0, &mut game_name_bytes)?;
    debug!(
        "read_bytes: {}, game_name_len: {}",
        read_bytes,
        game_name_bytes.len()
    );

    // remove null terminator
    let game_name = String::from_utf8_lossy(&game_name_bytes[0..read_bytes - 1]).to_string();

    debug!("game name: {}", game_name);
    debug!(
        "cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );

    debug!(
        "cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let mut c = [0u8; 6];
    Read::read_exact(&mut cursor, &mut c)?;
    debug!("reading timer");
    let d = cursor.read_i32::<LittleEndian>()?;
    debug!("timer value: {}", d);

    // let e = cursor.read_u8()?;

    debug!(
        "f cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let mut f = vec![0u8; 750];
    cursor.read_exact(&mut f)?;
    debug!(
        "g cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let g = cursor.read_u8()?;
    debug!(
        "h cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let h = cursor.read_u32::<LittleEndian>()?;
    debug!(
        "i cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let i = cursor.read_u32::<LittleEndian>()?;
    debug!(
        "j cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let j = cursor.read_u8()?;
    debug!(
        "finish cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );

    Ok(RawGameData {
        a,
        game_name,
        c,
        d,
        // e: e,
        f,
        g,
        h,
        i,
        j,
    })
}

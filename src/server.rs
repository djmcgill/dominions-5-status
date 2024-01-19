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
    let header_element = header_element.trim();

    if header_element == "Game is being setup" {
        return Ok(GameData {
            game_name: "".to_owned(),
            turn: -1, // TODO: this is a horrible hack to contort the dom6 data back into the dom5 form
            turn_deadline: Utc::now(),
            nations: vec![],
        });
    }

    let (game_name, turn, option_time_remaining, finished) =
        parse_header(&header_element).context("parse_header")?;

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

    let turn_deadline = Utc::now() + option_time_remaining.unwrap_or_default();

    Ok(GameData {
        game_name,
        nations,
        turn,
        turn_deadline,
    })
}

fn parse_header(header_element: &str) -> anyhow::Result<(String, i32, Option<Duration>, bool)> {
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
    let remaining_header = remaining_header.collect::<String>();
    let remaining_header = if remaining_header.contains(':') {
        remaining_header
            .split(':')
            .skip(1)
            .next()
            .expect("we just tested it had a ':' in it")
    } else {
        &remaining_header
    };

    // 1 days and 1 hours
    // okay I'm a bit worried we might see "1 days" or "1 day" or "1 hour" or "1 hours"
    // depending on if there's e.g. exactly 24 hours or <24 hours remaining
    let mut words = remaining_header
        .split(' ')
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>();

    let (finished, option_time_remaining) = if words.len() == 1 && words[0].contains("finished") {
        (true, None)
    } else if words.len() == 0 {
        (false, None)
    } else {
        let number_unit_pairs = if words.len() == 2 {
            vec![(words[0], words[1])]
        } else if words.len() == 5 && words[2] == "and" {
            vec![(words[0], words[1]), (words[3], words[4])]
        } else {
            return Err(anyhow!("Unknown duration: {}", remaining_header));
        };
        let seconds: i32 = number_unit_pairs
            .into_iter()
            .map(|(count, unit)| text_to_seconds(count, unit))
            .collect::<anyhow::Result<Vec<_>>>()
            .context("parse duration text")?
            .into_iter()
            .sum();
        (false, Some(Duration::from_secs(seconds as u64)))
    };

    Ok((game_name, turn, option_time_remaining, finished))
}

fn text_to_seconds(count: &str, unit: &str) -> anyhow::Result<i32> {
    let mult = if unit.contains("week") {
        7 * 24 * 60 * 60
    } else if unit.contains("day") {
        24 * 60 * 60
    } else if unit.contains("hour") {
        60 * 60
    } else if unit.contains("minute") {
        60
    } else if unit.contains("second") {
        1
    } else {
        return Err(anyhow!("Unknown time unit: '{}'", unit));
    };

    let count = i32::from_str(count).context("parse duration as number")?;
    Ok(mult * count)
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

#[cfg(test)]
mod test {
    use super::*;
    // samog, turn 41 (finished)
    // samog, turn 41
    // samog, turn 41 (time left: 1 days and 1 hours)
    // curtains, turn 1 (time left: 21 hours and 3 minutes)

    #[test]
    fn header_parse_no_timer() {
        let (game_name, turn, option_time_remaining, finished) =
            parse_header("samog, turn 41").unwrap();
        assert_eq!("samog", game_name);
        assert_eq!(41, turn);
        assert!(option_time_remaining.is_none());
        assert!(!finished);
    }

    #[test]
    fn header_parse_finished() {
        let (game_name, turn, option_time_remaining, finished) =
            parse_header("samog, turn 41 (finished)").unwrap();
        assert_eq!("samog", game_name);
        assert_eq!(41, turn);
        assert!(option_time_remaining.is_none());
        assert!(finished);
    }

    #[test]
    fn header_parse_timer_1() {
        let (game_name, turn, option_time_remaining, finished) =
            parse_header("samog, turn 41 (time left: 1 days and 1 hours)").unwrap();
        assert_eq!("samog", game_name);
        assert_eq!(41, turn);
        assert_eq!(
            Duration::from_secs(25 * 60 * 60),
            option_time_remaining.unwrap()
        );
        assert!(!finished);
    }

    #[test]
    fn header_parse_timer_2() {
        let (game_name, turn, option_time_remaining, finished) =
            parse_header("curtains, turn 1 (time left: 21 hours and 3 minutes)").unwrap();
        assert_eq!("curtains", game_name);
        assert_eq!(1, turn);
        assert_eq!(
            Duration::from_secs(21 * 60 * 60 + 3 * 60),
            option_time_remaining.unwrap()
        );
        assert!(!finished);
    }
}

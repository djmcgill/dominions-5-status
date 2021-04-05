use crate::model::{
    enums::{NationStatus, SubmissionStatus},
    game_data::GameData,
    nation::{GameNationIdentifier, Nation},
    raw_game_data::RawGameData,
};

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use log::*;
use std::io::{self, BufRead, Cursor, Read};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn get_game_data_async(server_address: &str) -> anyhow::Result<GameData> {
    let raw_data = get_raw_game_data_async(server_address).await?;
    let game_data = interpret_raw_data(raw_data)?;
    Ok(game_data)
}
fn interpret_raw_data(raw_data: RawGameData) -> anyhow::Result<GameData> {
    let mut game_data = GameData {
        game_name: raw_data.game_name,
        nations: vec![],
        turn: raw_data.h as i32,
        turn_timer: raw_data.d,
    };
    for i in 0..250 {
        let status_num = raw_data.f[i];
        if status_num != 0 && status_num != 3 {
            let submitted = raw_data.f[i + 250];
            let connected = raw_data.f[i + 500];
            let nation_id = (i - 1) as u32; // why -1? No fucking idea
            let nation = Nation {
                identifier: GameNationIdentifier::from_id(nation_id),
                status: NationStatus::from_int(status_num).ok_or(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Unknown nation status {}", status_num),
                ))?,
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

    let mut buffer = Vec::with_capacity(2048);
    debug!("trying to receive");
    let bytes_read = stream.read_to_end(&mut buffer).await?;
    debug!("received {} bytes", bytes_read);

    debug!("sending close");
    let close_request = [
        b'f', b'H', 0x01, 0x00, 0x00, 0x00, // ::<LittleEndian>(1)?;
        11,
    ];
    stream.write_all(&close_request).await?;

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
    let d = Read::read_i32::<LittleEndian>(&mut cursor)?;
    debug!("timer value: {}", d);

    // let e = cursor.read_u8()?;

    debug!(
        "f cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let mut f = vec![0u8; 750];
    Read::read_exact(&mut cursor, &mut f)?;
    debug!(
        "g cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let g = Read::read_u8(&mut cursor)?;
    debug!(
        "h cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let h = Read::read_u32::<LittleEndian>(&mut cursor)?;
    debug!(
        "i cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let i = Read::read_u32::<LittleEndian>(&mut cursor)?;
    debug!(
        "j cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    let j = Read::read_u8(&mut cursor)?;
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

use crate::model::enums::{NationStatus, Nations, SubmissionStatus};
use crate::snek::{snek_details, SnekGameStatus};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::ZlibDecoder;
use hex_slice::AsHex;
use log::*;
use std::error::Error;
use std::io;
use std::io::{Cursor, Read, Write, BufRead};
use std::net;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::time::Duration;
use crate::model::game_data::GameData;
use crate::model::raw_game_data::RawGameData;
use crate::model::nation::Nation;

pub trait ServerConnection {
    fn get_game_data(server_address: &str) -> io::Result<GameData>;
    fn get_snek_data(server_address: &str) -> Result<Option<SnekGameStatus>, Box<dyn Error>>;
}

fn get_game_data_cache(server_address: &str) -> io::Result<GameData> {
    let raw_data = get_raw_game_data(server_address)?;
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
            let &(nation_name, era) = Nations::get_nation_desc(nation_id);
            let nation = Nation {
                id: nation_id,
                status: NationStatus::from_int(status_num).ok_or(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Unknown nation status {}", status_num),
                ))?,
                submitted: SubmissionStatus::from_int(submitted),
                connected: connected == 1,
                name: nation_name.to_owned(),
                era: format!("{}", era),
            };
            game_data.nations.push(nation);
        }
    }
    Ok(game_data)
}

pub struct RealServerConnection;

impl ServerConnection for RealServerConnection {
    fn get_game_data(server_address: &str) -> io::Result<GameData> {
        get_game_data_cache(server_address)
    }
    fn get_snek_data(server_address: &str) -> Result<Option<SnekGameStatus>, Box<dyn Error>> {
        snek_details(server_address)
    }
}

fn get_raw_game_data(server_address: &str) -> io::Result<RawGameData> {
    let buffer = call_server_for_info(server_address)?;
    let decompressed = decompress_server_info(&buffer)?;
    let game_data = parse_data(&decompressed)?;
    Ok(game_data)
}

fn call_server_for_info(server_address: &str) -> io::Result<Vec<u8>> {
    info!("starting to connect to {}", server_address);
    let parsed_address: SocketAddr =
        server_address
            .to_socket_addrs()?
            .next()
            .ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Could not resolve ip address",
            ))?;
    let mut stream = net::TcpStream::connect_timeout(&parsed_address, Duration::from_secs(30))?;
    debug!("connected");
    // https://steamcommunity.com/app/722060/discussions/0/1749024748627269322/?ctp=2#c1749024925634051868
    // (b'f', b'H', b'\a', b'\x00', b'\x00',
    // b'\x00', b'=', b'\x1e', b'\x02', b'\x11', b'E', b'\x05', b'\x00')
    // b'<ccssssccccccc'
    let mut wtr = vec![];
    wtr.write_u8(b'f')?;
    wtr.write_u8(b'H')?;
    // wtr.write_u32::<LittleEndian>(1)?;
    // wtr.write_u8(3)?;
    wtr.write_u8(0x07)?;
    wtr.write_u8(0x00)?;
    wtr.write_u8(0x00)?;
    wtr.write_u8(0x00)?;
    wtr.write_u8(b'=')?;
    wtr.write_u8(0x1e)?;
    wtr.write_u8(0x02)?;
    wtr.write_u8(0x11)?;
    wtr.write_u8(b'E')?;
    wtr.write_u8(0x05)?;
    wtr.write_u8(0x00)?;

    debug!("Sending {:x}", wtr.as_slice().as_hex());
    let _ = stream.write(&wtr)?;
    debug!("sent");
    let mut buffer = [0; 2048];
    debug!("trying to receive");
    let _ = stream.read(&mut buffer)?;

    let mut wtr2 = vec![];
    wtr2.write_u8(b'f')?;
    wtr2.write_u8(b'H')?;
    wtr2.write_u32::<LittleEndian>(1)?;
    wtr2.write_u8(11)?;
    debug!("Sending {:x}", wtr2.as_slice().as_hex());
    let _ = stream.write(&wtr2)?;
    debug!("sent");

    Ok(buffer.to_vec())
}

fn decompress_server_info(raw: &[u8]) -> io::Result<Vec<u8>> {
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

fn parse_data(data: &[u8]) -> io::Result<RawGameData> {
    let len = data.len();
    debug!("done: data.len(): {}", len);

    let mut cursor = Cursor::new(data);
    let mut a = [0u8; 6];
    cursor.read_exact(&mut a)?;
    debug!(
        "cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    debug!("parsing name");
    let mut game_name_bytes = vec![];
    let read_bytes = cursor.read_until(0, &mut game_name_bytes)?;
    debug!("read_bytes: {}, game_name_len: {}", read_bytes, game_name_bytes.len());

    // remove null terminator
    let game_name = String::from_utf8_lossy(&game_name_bytes[0..read_bytes-1]).to_string();

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
    cursor.read_exact(&mut c)?;
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

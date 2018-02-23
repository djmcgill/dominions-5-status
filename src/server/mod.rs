use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use cached::TimedCache;
use hex_slice::AsHex;
use flate2::read::ZlibDecoder;
use std::io::{Cursor, Read, Write};
use std::io;
use std::net;
use model::{GameData, Nation, RawGameData};
use model::enums::{NationStatus, Nations, SubmissionStatus};

pub trait ServerConnection {
    fn get_game_data(server_address: &str) -> io::Result<GameData>;
}


cached_key_result! {
    ONE_MIN_GAME_DATA: TimedCache<String, GameData> = TimedCache::with_lifespan(59);
    Key = { server_address.to_owned() };
    fn get_game_data_fn(server_address: &str) -> io::Result<GameData> = {
        get_game_data_cache(server_address)
    }
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
            let nation_id = i - 1; // why -1? No fucking idea
            let &(nation_name, era) = Nations::get_nation_desc(nation_id);
            let nation = Nation {
                id: nation_id,
                status: NationStatus::from_int(status_num),
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
        get_game_data_fn(server_address)
    }
}

fn get_raw_game_data(server_address: &str) -> io::Result<RawGameData> {
    let buffer = call_server_for_info(server_address)?;
    let decompressed = decompress_server_info(&buffer)?;
    let game_data = parse_data(&decompressed)?;
    Ok(game_data)
}

fn call_server_for_info(server_address: &str) -> io::Result<Vec<u8>> {
    info!("starting");
    let mut stream = net::TcpStream::connect(server_address)?;
    info!("connected");
    let mut wtr = vec![];
    wtr.write_u8(b'f')?;
    wtr.write_u8(b'H')?;
    wtr.write_u32::<LittleEndian>(1)?;
    wtr.write_u8(3)?;

    info!("Sending {:x}", wtr.as_slice().as_hex());
    let _ = stream.write(&mut wtr)?;
    info!("sent");
    let mut buffer = [0; 2048];
    info!("trying to receive");
    let _ = stream.read(&mut buffer)?;

    let mut wtr2 = vec![];
    wtr2.write_u8(b'f')?;
    wtr2.write_u8(b'H')?;
    wtr2.write_u32::<LittleEndian>(1)?;
    wtr2.write_u8(11)?;
    info!("Sending {:x}", wtr2.as_slice().as_hex());
    let _ = stream.write(&mut wtr2)?;
    info!("sent");

    Ok(buffer.to_vec())
}

fn decompress_server_info(raw: &[u8]) -> io::Result<Vec<u8>> {
    if raw[1] == b'J' {
        info!("decompressing");
        let mut decoder = ZlibDecoder::new(&raw[10..]);
        let mut decompressed = vec![];
        let _ = decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    } else {
        info!("No need to decompress");
        Ok(raw[10..].to_vec())
    }
}

fn parse_data(data: &[u8]) -> io::Result<RawGameData> {
    let game_name_len = data.len() - 26 - 750;

    let mut cursor = Cursor::new(data);
    let mut a = [0u8; 6];
    cursor.read(&mut a)?;
    debug!(
        "cursor position: {}, cursor len: {}",
        cursor.position(),
        cursor.get_ref().len()
    );
    debug!("parsing name");
    // TODO: properly read until null terminator instead of this hack
    let mut game_name_buff = vec![0u8; game_name_len];
    let _ = cursor.read_exact(&mut game_name_buff)?;
    let game_name = String::from_utf8_lossy(&game_name_buff[0..game_name_len - 1]).to_string();
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
    cursor.read(&mut c)?;
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
        a: a,
        game_name: game_name,
        c: c,
        d: d,
        // e: e,
        f: f,
        g: g,
        h: h,
        i: i,
        j: j,
    })
}

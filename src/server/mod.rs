use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use hex_slice::AsHex;
use flate2::read::ZlibDecoder;
use std::io::{Cursor, Read, Write};
use std::io;
use std::net;
use model::raw_game_data::RawGameData;
use model::game_data::GameData;
use model::nation::Nation;
use model::enums::nations;
use model::enums::submission_status::SubmissionStatus;
use model::enums::nation_status::NationStatus;

pub fn get_game_data(server_address: &String) -> io::Result<GameData> {
    let raw_data = get_raw_game_data(server_address)?;
    let mut game_data = GameData {
        game_name: raw_data.game_name,
        nations: vec![],
        turn: raw_data.h as i32,
    };
    for i in 0..250 {
        let status_num = raw_data.f[i];        
        if status_num != 0 && status_num != 3 {
            let submitted = raw_data.f[i+250];
            let connected = raw_data.f[i+500];
            let nation_id = i-1; // why -1? No fucking idea
            let &(nation_name, era) = nations::get_nation_desc(nation_id); 
            let nation = Nation {
                id: nation_id,
                status: NationStatus::from_int(status_num),
                submitted: SubmissionStatus::from_int(submitted),
                connected: connected == 1,
                name: nation_name.to_string(),
                era: era.to_string(),
            };
            game_data.nations.push(nation);
        }
    }
    Ok(game_data)
}

pub fn get_raw_game_data(server_address: &String) -> io::Result<RawGameData> {
    let buffer = call_server_for_info(server_address)?;
    let decompressed = decompress_server_info(&buffer)?;
    let game_data = parse_data(&decompressed)?;
    println!("data: {:?}", game_data);
    Ok(game_data)
}

fn call_server_for_info(server_address: &String) -> io::Result<Vec<u8>> {
    println!("starting");
    let mut stream = net::TcpStream::connect(server_address)?;
    println!("connected");
    let mut wtr = vec![];
    wtr.write_u8(b'f')?;
    wtr.write_u8(b'H')?;
    wtr.write_u32::<LittleEndian>(1)?;
    wtr.write_u8(3)?;

    println!("Sending {:x}", wtr.as_slice().as_hex());
    let _ = stream.write(&mut wtr)?;
    println!("sent");
    let mut buffer = [0; 512];
    println!("trying to receive");
    let _ = stream.read(&mut buffer)?;
    println!("Received {:x}", buffer.as_hex());

    let mut wtr2 = vec![];
    wtr2.write_u8(b'f')?;
    wtr2.write_u8(b'H')?;
    wtr2.write_u32::<LittleEndian>(1)?;
    wtr2.write_u8(11)?;
    println!("Sending {:x}", wtr2.as_slice().as_hex());
    let _ = stream.write(&mut wtr2)?;
    println!("sent");

    Ok(buffer.to_vec())
}

fn decompress_server_info(raw: &[u8]) -> io::Result<Vec<u8>> {
    if raw[1] == b'J' {
        let mut decoder = ZlibDecoder::new(&raw[10..]);
        let mut decompressed = vec![];
        let _ = decoder.read_to_end(&mut decompressed)?;
        println!("{:x}", decompressed.as_slice().as_hex());
        Ok(decompressed)
    } else {
        Ok(raw[10..].to_vec())
    }
}

fn parse_data(data: &[u8]) -> io::Result<RawGameData> {
    let game_name_len = data.len() - 27 - 750; // Possibly null terminated?
    let mut cursor = Cursor::new(data);
    let mut a = [0u8; 6]; 
    cursor.read(&mut a)?;
    
    let mut game_name_buff = vec![0u8; game_name_len];
    cursor.read_exact(&mut game_name_buff)?;
    let game_name = String::from_utf8_lossy(&game_name_buff).to_string();

    let mut c = [0u8; 6];
    cursor.read(&mut c)?;
    
    let d = cursor.read_u32::<LittleEndian>()?;

    let e = cursor.read_u8()?;

    let mut f = vec![0u8; 750];
    cursor.read_exact(&mut f)?;

    let g = cursor.read_u8()?;

    let h = cursor.read_u32::<LittleEndian>()?;

    let i = cursor.read_u32::<LittleEndian>()?;

    let j = cursor.read_u8()?;
    assert!(cursor.position() as usize == cursor.get_ref().len());

    Ok(RawGameData {
        a: a,
        game_name: game_name,
        c: c,
        d: d,
        e: e,
        f: f,
        g: g,
        h: h,
        i: i,
        j: j,
    })
}

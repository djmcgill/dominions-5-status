extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

extern crate hex_slice;
use hex_slice::AsHex;

extern crate flate2;
use flate2::read::ZlibDecoder;

use std::fs::File;
use std::io;
use std::net;
use std::io::{Cursor, Read, Write};

#[macro_use]
extern crate serenity;
use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::standard::StandardFramework;

mod nations;

struct Handler;
impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

command!(ping(_context, message) {
    println!{"ping message: {:?}", message};
    let _ = message.reply("Pong!");
});

command!(game_name(_context, message, args) {
    println!{"game_name message: {:?}", message};
    let server_address = args.single::<String>().unwrap();
    let response = get_game_data(&server_address).unwrap().game_name;
    let _ = message.reply(&format!("Game name at {} is {}", server_address, response));
});

command!(nation_status(_context, message, args) {
    println!{"nation_status message: {:?}", message};
    let server_address = args.single::<String>().unwrap();
    let data = get_game_data(&server_address).unwrap();
    let mut response = String::new();
    for i in 0..250 {
        let status_num = data.f[i];        
        if status_num != 0 && status_num != 3 {
            let submitted = data.f[i+250];
            let connected = data.f[i+500];
            let nation_name = nations::get_nation_desc(i-1); // why -1? No fucking idea
            response.push_str(&format!(
                "name: {}, status: {}, submitted: {}, connected: {}\n", nation_name, status_num, submitted, connected
            ))
        }
    }
    println!("responding with {}", response);
    let _ = message.reply(&response);    
});

fn main() {
    let token = {
        let mut token_file = File::open("token").unwrap();
        let mut temp_token = String::new();
        token_file.read_to_string(&mut temp_token).unwrap();
        temp_token
    };
    
    let mut client = Client::new(&token, Handler);
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .on("ping", ping)
        .on("game_name", game_name)
        .on("nation_status", nation_status));

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
    println!("returning");
}

fn decompress_server_info(raw: &[u8]) -> io::Result<Vec<u8>> {
    assert!(raw[1] == b'J');
    let mut decoder = ZlibDecoder::new(&raw[10..]);
    let mut decompressed = vec![];
    let _ = decoder.read_to_end(&mut decompressed)?;
    println!("{:x}", decompressed.as_slice().as_hex());
    Ok(decompressed)
}

fn get_game_data(server_address: &String) -> io::Result<RawGameData> {
    let buffer = call_server_for_info(server_address)?;
    let decompressed = decompress_server_info(&buffer)?;
    let game_data = parse_data(&decompressed)?;
    println!("data: {:?}", game_data);
    Ok(game_data)
}

fn call_server_for_info(server_address: &String) -> io::Result<Vec<u8>> {
    println!("starting");
    let mut stream = net::TcpStream::connect(server_address)?;
    // let mut stream = net::TcpStream::connect("dom5.snek.earth:30028")?;
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

// gamenamelength = len(data) - len(PACKET_GENERAL_INFO.format("", "")) - PACKET_BYTES_PER_NATION * PACKET_NUM_NATIONS - 6
// dataArray = struct.unpack(PACKET_GENERAL_INFO.format(gamenamelength, PACKET_BYTES_PER_NATION * PACKET_NUM_NATIONS), data)
    // PACKET_BYTES_PER_NATION = 3
// PACKET_NUM_NATIONS = 250
// PACKET_GENERAL_INFO = '<BBBBBB{0}sBBBBBBLB{1}BLLB'  # to use format later
// PACKET_NATION_INFO_START = 15
// fn parse_server_info_for_game_name(unzipped_info: &[u8]) -> io::Result<String> {
//     let game_name_len = unzipped_info.len() - 27 - 750;
//     println!("name len {}", game_name_len);
//     let game_name_bytes: &[u8] = &unzipped_info[6..6+game_name_len];
//     let game_name = String::from_utf8_lossy(game_name_bytes);
//     Ok(game_name.to_string())
// }
#[repr(C)]
#[derive(Debug)]
struct RawGameData {
    a: [u8; 6], // 6
    game_name: String,
    c: [u8; 6], // 6
    d: u32, // 4
    e: u8,  // 1
    f: Vec<u8>, // ; 750],
    g: u8,  // 1
    h: u32, // 4
    i: u32, // 4
    j: u8,  // 1
}
fn parse_data(data: &[u8]) -> io::Result<RawGameData> {
    let game_name_len = data.len() - 27 - 750; // Possibly null terminated?
    let mut cursor = Cursor::new(data);
    let mut a = [0u8; 6]; 
    cursor.read(&mut a).unwrap();
    
    let mut game_name_buff = vec![0u8; game_name_len];
    cursor.read_exact(&mut game_name_buff).unwrap();
    let game_name = String::from_utf8_lossy(&game_name_buff).to_string();

    let mut c = [0u8; 6];
    cursor.read(&mut c).unwrap();
    
    let d = cursor.read_u32::<LittleEndian>().unwrap();

    let e = cursor.read_u8().unwrap();

    let mut f = vec![0u8; 750];
    cursor.read_exact(&mut f).unwrap();

    let g = cursor.read_u8().unwrap();

    let h = cursor.read_u32::<LittleEndian>().unwrap();

    let i = cursor.read_u32::<LittleEndian>().unwrap();

    let j = cursor.read_u8().unwrap();
    assert!(cursor.position() as usize == cursor.get_ref().len());

    for i in 0..250 {
        let status_num = f[i];        
        if status_num != 0 && status_num != 3 {
            let submitted = f[i+250];
            let connected = f[i+500];
            println!("i: {}", i);
            println!("status_num: {}", status_num);
            println!("nation_desc: {}", nations::get_nation_desc(i-1));
            println!("submitted: {}", submitted);
            println!("connected: {}", connected);
            println!("--------")
        }
    }

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

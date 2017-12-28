// const TEST_URL: &'static str = "snek.earth";
// const TEST_ADDR: &'static str = "91.105.250.132";
// const TEST_PORT: u32 = 30013;


extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};

extern crate hex_slice;
use hex_slice::AsHex;

extern crate flate2;
use flate2::read::ZlibDecoder;

use std::fs::File;
use std::io;
use std::net;
use std::io::{Read, Write};

#[macro_use]
extern crate serenity;
use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::standard::StandardFramework;

// const PACKET_HEADER: &'static str = "<ccLB";

struct Handler;
impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

command!(ping(_context, message) {
    println!{"message: {:?}", message};
    let _ = message.reply("Pong!");
});

command!(game_name(_context, message, args) {
    println!{"message: {:?}", message};
    let server_address = args.single::<String>().unwrap();
    let response = get_game_name(&server_address).unwrap();
    let _ = message.reply(&format!("Game name at {} is {}", server_address, response));
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
        .on("game_name", game_name));

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

fn get_game_name(server_address: &String) -> io::Result<String> {
    let buffer = call_server_for_info(server_address)?;
    let decompressed = decompress_server_info(&buffer)?;
    let game_name = parse_server_info_for_game_name(&decompressed)?;
    println!("game name: {}", game_name);
    Ok(game_name)
}

// PACKET_BYTES_PER_NATION = 3
// PACKET_NUM_NATIONS = 250
// PACKET_GENERAL_INFO = '<BBBBBB{0}sBBBBBBLB{1}BLLB'  # to use format later
// PACKET_NATION_INFO_START = 15
fn parse_server_info_for_game_name(unzipped_info: &[u8]) -> io::Result<String> {
    let game_name_len = unzipped_info.len() - 20 - 6 - 750;
    println!("name len {}", game_name_len);
    let game_name_bytes: &[u8] = &unzipped_info[6..6+game_name_len];
    let game_name = String::from_utf8_lossy(game_name_bytes);
    Ok(game_name.to_string())
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

// const TEST_URL: &'static str = "snek.earth";
// const TEST_ADDR: &'static str = "91.105.250.132";
// const TEST_PORT: u32 = 30013;

extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};

extern crate hex_slice;
use hex_slice::AsHex;

extern crate flate2;
use flate2::read::ZlibDecoder;

use std::io;
use std::net;
use std::io::{Read, Write};

// const PACKET_HEADER: &'static str = "<ccLB";

fn main() {
    do_main().unwrap();
}

fn do_main() -> io::Result<()> {
    let buffer = call_server_for_info()?;
    let compressed = buffer[1] == b'J';
    assert!(compressed);
    println!("starting decoding");
    let mut decoder = ZlibDecoder::new(&buffer[10..]);
    let mut decompressed = vec![];
    let _ = decoder.read_to_end(&mut decompressed)?;
    println!("{:x}", decompressed.as_slice().as_hex());
    println!("{}", String::from_utf8_lossy(&decompressed));
    Ok(())
}

fn call_server_for_info() -> io::Result<Vec<u8>> {
    println!("starting");
    let mut stream = net::TcpStream::connect("91.105.250.132:30013")?;
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

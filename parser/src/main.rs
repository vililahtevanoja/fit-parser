use byteorder::{ByteOrder, LittleEndian};
use fit_parser::fit_header::{fit_crc, parse_fit_header_from_data};

use std::io::Read;

fn main() {
    let input_path = "../7427193981_ACTIVITY.fit";
    let mut input_file = std::fs::File::open(input_path).unwrap();
    let data_size = input_file.metadata().unwrap().len() as usize;
    println!("File size: {}", data_size);
    let mut fit_content: Vec<u8> = vec![0; data_size];

    input_file.read_exact(&mut *fit_content).unwrap();

    let header = parse_fit_header_from_data(&fit_content);

    let file_crc_slice = &fit_content[data_size - 2..];
    let file_crc = LittleEndian::read_u16(file_crc_slice);
    let calculated_file_crc = fit_crc(&fit_content[..data_size - 2], 0);
    println!(
        "CRC in file: {}, calculated CRC: {}",
        file_crc, calculated_file_crc
    );
    assert_eq!(file_crc, calculated_file_crc);
    println!("Header: {:#?}", header);
}

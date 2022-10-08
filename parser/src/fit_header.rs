use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FitFileHeader {
    header_size: u8,
    protocol_version: u8,
    profile_version: u16,
    data_size: u32,
    data_type: String,
    crc: Option<u16>,
}

struct FitFileHeaderOffsets {}

impl FitFileHeaderOffsets {
    pub const HEADER_SIZE: usize = 0;
    const PROTOCOL_VERSION: usize = 1;
    const PROFILE_VERSION_LSB: usize = 2;
    const PROFILE_VERSION_MSB: usize = 3;
    const DATA_SIZE_LSB: usize = 4;
    const DATA_SIZE_MSB: usize = 7;
    const DATA_TYPE_START: usize = 8;
    const DATA_TYPE_END: usize = 11;
    const CRC_LSB: usize = 12;
    const CRC_MSB: usize = 13;
}

pub fn parse_fit_header_from_data(fit_data: &[u8]) -> FitFileHeader {
    FitFileHeader::from(fit_data)
}

impl FitFileHeader {
    fn from(fit_data: &[u8]) -> FitFileHeader {
        let header_size: u8 = fit_data[FitFileHeaderOffsets::HEADER_SIZE];
        assert!(header_size as usize <= fit_data.len());
        let protocol_version: u8 = fit_data[FitFileHeaderOffsets::PROTOCOL_VERSION];
        let profile_version: u16 = LittleEndian::read_u16(
            &fit_data[FitFileHeaderOffsets::PROFILE_VERSION_LSB
                ..=FitFileHeaderOffsets::PROFILE_VERSION_MSB],
        );
        assert_eq!(
            FitFileHeaderOffsets::DATA_SIZE_MSB - FitFileHeaderOffsets::DATA_SIZE_LSB,
            3
        );
        let data_size = LittleEndian::read_u32(
            &fit_data[FitFileHeaderOffsets::DATA_SIZE_LSB..=FitFileHeaderOffsets::DATA_SIZE_MSB],
        );
        let data_type = String::from_utf8(
            fit_data[FitFileHeaderOffsets::DATA_TYPE_START..=FitFileHeaderOffsets::DATA_TYPE_END]
                .to_vec(),
        )
        .unwrap();
        let mut crc: Option<u16> = None;

        if header_size > (FitFileHeaderOffsets::CRC_LSB as u8)
            && header_size >= (FitFileHeaderOffsets::CRC_MSB as u8)
        {
            let crc_in_data = LittleEndian::read_u16(
                &fit_data[FitFileHeaderOffsets::CRC_LSB..=FitFileHeaderOffsets::CRC_MSB],
            );
            let calculated_crc = fit_crc_vec(fit_data[..FitFileHeaderOffsets::CRC_LSB].to_vec(), 0);
            assert_eq!(crc_in_data, calculated_crc, "crc mismatch");
            crc = Some(crc_in_data);
        }

        FitFileHeader {
            header_size,
            protocol_version,
            profile_version,
            data_size,
            data_type,
            crc,
        }
    }
}

static CRC_TABLE: [u16; 16] = [
    0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401, 0xA001, 0x6C00, 0x7800, 0xB401,
    0x5000, 0x9C01, 0x8801, 0x4400,
];

pub fn fit_crc(data: &[u8], crc_in: u16) -> u16 {
    let mut crc = crc_in;
    let mut tmp: u16;
    for byte in data {
        tmp = CRC_TABLE[(crc & 0xF) as usize];
        crc = (crc >> 4) & 0x0FFF;
        crc = crc ^ tmp ^ CRC_TABLE[(byte & 0xF) as usize];

        tmp = CRC_TABLE[(crc & 0xF) as usize];
        crc = (crc >> 4) & 0x0FFF;
        crc = crc ^ tmp ^ CRC_TABLE[((byte >> 4) & 0xF) as usize];
    }
    crc
}

fn fit_crc_vec(data: Vec<u8>, crc_in: u16) -> u16 {
    let mut crc = crc_in;
    let mut tmp: u16;
    for byte in data {
        tmp = CRC_TABLE[(crc & 0xF) as usize];
        crc = (crc >> 4) & 0x0FFF;
        crc = crc ^ tmp ^ CRC_TABLE[(byte & 0xF) as usize];

        tmp = CRC_TABLE[(crc & 0xF) as usize];
        crc = (crc >> 4) & 0x0FFF;
        crc = crc ^ tmp ^ CRC_TABLE[((byte >> 4) & 0xF) as usize];
    }
    crc
}

#[test]
fn fit_file_header_from_data() {
    let expected_header = FitFileHeader {
        header_size: 14,
        protocol_version: 3,
        profile_version: 0x0A0B,
        data_size: 0x0A0B0C0D,
        data_type: String::from(".FIT"),
        crc: Some(0xA7A3),
    };
    let data: Vec<u8> = vec![
        14, // header size
        3,  // protocol version
        0x0B, 0x0A, // profile version
        0x0D, 0x0C, 0x0B, 0x0A, // data_size
        0x2E, 0x46, 0x49, 0x54, // data type
        0xA3, 0xA7, // crc
        0xA3, 0xA7, // file crc
    ];
    let actual_header = FitFileHeader::from(&data);
    assert_eq!(
        expected_header, actual_header,
        "expected: {:?}, actual: {:?}",
        expected_header, actual_header
    )
}

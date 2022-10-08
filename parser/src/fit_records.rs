use byteorder::{BigEndian, ByteOrder, LittleEndian};
use num_enum::{IntoPrimitive, TryFromPrimitive};

include!(concat!(env!("OUT_DIR"), "/fit.rs"));

#[derive(Debug, Clone, PartialEq)]
enum Endianness {
    BigEndian,
    LittleEndian,
}

#[derive(Debug, Clone, PartialEq)]
struct BaseTypeInfo {
    base_type: BaseType,
    endian_ability: bool,
    base_type_field: u8,
    type_name: String,
    size: u8,
    invalid_value: u64,
}

#[derive(Debug, Eq, Clone, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum BaseType {
    Enum = 0x00,
    Sint8 = 0x01,
    Uint8 = 0x02,
    Sint16 = 0x83,
    Uint16 = 0x84,
    Sint32 = 0x85,
    Uint32 = 0x86,
    String = 0x07,
    Float32 = 0x88,
    Float64 = 0x89,
    Uint8z = 0x0A,
    Uint16z = 0x8B,
    Uint32z = 0x8C,
    Byte = 0x0D,
    Sint64 = 0x8E,
    Uint64 = 0x8F,
    Uint64z = 0x90,
}

fn get_base_type_info(number: u8) -> BaseTypeInfo {
    let base_type = BaseType::try_from(number).unwrap();
    match base_type {
        BaseType::Enum => BaseTypeInfo {
            base_type,
            endian_ability: false,
            base_type_field: 0x00,
            type_name: String::from("enum"),
            invalid_value: 0xFF,
            size: 1,
        },
        BaseType::Sint8 => BaseTypeInfo {
            base_type,
            endian_ability: false,
            base_type_field: 0x01,
            type_name: String::from("sint8"),
            invalid_value: 0x7F,
            size: 1,
        },
        BaseType::Uint8 => BaseTypeInfo {
            base_type,
            endian_ability: false,
            base_type_field: 0x02,
            type_name: String::from("uint8"),
            invalid_value: 0xFF,
            size: 1,
        },
        BaseType::Sint16 => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x83,
            type_name: String::from("sint16"),
            invalid_value: 0x7FFF,
            size: 2,
        },
        BaseType::Uint16 => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x84,
            type_name: String::from("uint16"),
            invalid_value: 0xFFFF,
            size: 2,
        },
        BaseType::Sint32 => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x85,
            type_name: String::from("sint32"),
            invalid_value: 0x7FFFFFFF,
            size: 4,
        },
        BaseType::Uint32 => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x86,
            type_name: String::from("uint32"),
            invalid_value: 0xFFFFFFFF,
            size: 4,
        },
        BaseType::String => BaseTypeInfo {
            base_type,
            endian_ability: false,
            base_type_field: 0x07,
            type_name: String::from("string"),
            invalid_value: 0x00,
            size: 1,
        },
        BaseType::Float32 => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x88,
            type_name: String::from("float32"),
            invalid_value: 0xFFFFFFFF,
            size: 4,
        },
        BaseType::Float64 => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x89,
            type_name: String::from("float64"),
            invalid_value: 0xFFFFFFFFFFFFFFFF,
            size: 8,
        },
        BaseType::Uint8z => BaseTypeInfo {
            base_type,
            endian_ability: false,
            base_type_field: 0x0A,
            type_name: String::from("uint8z"),
            invalid_value: 0x00,
            size: 1,
        },
        BaseType::Uint16z => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x8B,
            type_name: String::from("uint16z"),
            invalid_value: 0x0000,
            size: 2,
        },
        BaseType::Uint32z => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x8C,
            type_name: String::from("uint32z"),
            invalid_value: 0x00000000,
            size: 4,
        },
        BaseType::Byte => BaseTypeInfo {
            base_type,
            endian_ability: false,
            base_type_field: 0x0D,
            type_name: String::from("byte"),
            invalid_value: 0xFF,
            size: 1,
        },
        BaseType::Sint64 => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x8E,
            type_name: String::from("sint64"),
            invalid_value: 0x7FFFFFFFFFFFFFFF,
            size: 8,
        },
        BaseType::Uint64 => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x8F,
            type_name: String::from("uint64"),
            invalid_value: 0xFFFFFFFFFFFFFFFF,
            size: 8,
        },
        BaseType::Uint64z => BaseTypeInfo {
            base_type,
            endian_ability: true,
            base_type_field: 0x90,
            type_name: String::from("uint64z"),
            invalid_value: 0x0000000000000000,
            size: 8,
        },
    }
}

#[derive(Debug, Clone, PartialEq)]
struct NormalDefinitionHeader {
    contains_extended_definitions: bool,
    local_message_type: u8,
}

#[derive(Debug, Clone, PartialEq)]
struct NormalDataHeader {
    local_message_type: u8,
}

#[derive(Debug, Clone, PartialEq)]
struct CompressedTimestampHeader {
    local_message_type: u8,
    time_offset: u8,
}

#[derive(Debug, Clone, PartialEq)]
enum RecordHeader {
    NormalDefinition(NormalDefinitionHeader),
    NormalData(NormalDataHeader),
    CompressedTimestamp(CompressedTimestampHeader),
}

impl RecordHeader {
    fn get_record_message_type(&self) -> RecordMessageType {
        match self {
            Self::NormalDefinition(NormalDefinitionHeader {
                contains_extended_definitions: _,
                local_message_type: _,
            }) => RecordMessageType::Definition,
            Self::NormalData(NormalDataHeader {
                local_message_type: _,
            }) => RecordMessageType::Data,
            Self::CompressedTimestamp(CompressedTimestampHeader {
                local_message_type: _,
                time_offset: _,
            }) => RecordMessageType::DataCompressedTimestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FieldDefinition {
    field_definition_number: u8,
    field_size: u8,
    base_type: BaseTypeInfo,
}

#[derive(Debug, Clone, PartialEq)]
struct DeveloperFieldDefinition {
    field_number: u8,
    field_size: u8,
    developer_data_index: u8,
}

#[derive(Debug, Clone, PartialEq)]
struct DefinitionRecord {
    header: NormalDefinitionHeader,
    architecture: Endianness,
    global_message_number: u16,
    field_definitions: Vec<FieldDefinition>,
    developer_field_definitions: Vec<DeveloperFieldDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
enum RecordMessageType {
    Definition,
    Data,
    DataCompressedTimestamp,
}

#[derive(Debug, Clone, PartialEq)]
struct RecordDefinitionHeaderMessage {
    message_type: RecordMessageType,
    local_message_type: u8,
}

#[derive(Debug, Clone, PartialEq)]
struct DeveloperDataIdMessage {
    application_id: u128,
    developer_data_index: u8,
}

#[derive(Debug, Clone, PartialEq)]
struct FieldDescriptionMessage {
    developer_data_index: u8,
    field_definition_number: u8,
    fit_base_type_id: u8,
    field_name: String,
    units: String,
    native_field_num: u8,
}

fn parse_record_header(b: u8) -> RecordHeader {
    if b & 0b10000000 > 0 {
        // is compressed timestamp header and message
        let local_message_type = b & 0b01100000;
        let time_offset = b & 0b00011111;
        return RecordHeader::CompressedTimestamp(CompressedTimestampHeader {
            local_message_type,
            time_offset,
        });
    }
    if b & 0b01000000 > 0 {
        // is definition message header
        let contains_extended_definitions = b & 0b00100000 > 0;
        let local_message_type = b & 0b00001111;
        RecordHeader::NormalDefinition(NormalDefinitionHeader {
            contains_extended_definitions,
            local_message_type,
        })
    } else {
        let local_message_type = b & 0b00001111;
        RecordHeader::NormalData(NormalDataHeader { local_message_type })
    }
}

#[test]
fn test_parse_record_header() {
    assert_eq!(
        parse_record_header(0b10000000),
        RecordHeader::CompressedTimestamp(CompressedTimestampHeader {
            local_message_type: 0,
            time_offset: 0
        })
    );
    assert_eq!(
        parse_record_header(0b10000000).get_record_message_type(),
        RecordMessageType::DataCompressedTimestamp
    );
    assert_eq!(
        parse_record_header(0b01000000),
        RecordHeader::NormalDefinition(NormalDefinitionHeader {
            contains_extended_definitions: false,
            local_message_type: 0
        })
    );
    assert_eq!(
        parse_record_header(0b01000000).get_record_message_type(),
        RecordMessageType::Definition
    );
    assert_eq!(
        parse_record_header(0b00000000),
        RecordHeader::NormalData(NormalDataHeader {
            local_message_type: 0
        })
    );
    assert_eq!(
        parse_record_header(0b00000000).get_record_message_type(),
        RecordMessageType::Data
    );
}

fn parse_definition_record(
    data: &[u8],
    header: NormalDefinitionHeader,
    data_start_offset: usize,
) -> (DefinitionRecord, usize) {
    let mut curr_idx = data_start_offset + 1; // skip first reserved byte

    let architecture = if data[curr_idx] > 0 {
        Endianness::BigEndian
    } else {
        Endianness::LittleEndian
    };
    curr_idx += 1;

    let global_message_number = match architecture {
        Endianness::LittleEndian => LittleEndian::read_u16(&data[curr_idx..=curr_idx + 1]),
        Endianness::BigEndian => BigEndian::read_u16(&data[curr_idx..=curr_idx + 1]),
    };
    curr_idx += 2;

    let number_of_fields = data[curr_idx];
    curr_idx += 1;

    let mut fields: Vec<FieldDefinition> = vec![];
    for _ in 0..number_of_fields {
        let field_definition_number = data[curr_idx];
        let size = data[curr_idx + 1];
        let base_type_number = data[curr_idx + 2];
        let base_type = get_base_type_info(base_type_number);
        if size % base_type.size != 0 {
            panic!(
                "Size {:?} of message field not multiple of base type {:?} size {:?}",
                size, base_type.type_name, base_type.size
            );
        }
        fields.push(FieldDefinition {
            field_definition_number,
            field_size: size,
            base_type,
        });
        curr_idx += 3;
    }

    let mut developer_fields: Vec<DeveloperFieldDefinition> = vec![];
    if header.contains_extended_definitions {
        let number_of_developer_fields = data[curr_idx];
        curr_idx += 1;
        for _ in 0..number_of_developer_fields {
            let field_number = data[curr_idx];
            let size = data[curr_idx + 1];
            let developer_data_index = data[curr_idx + 2];
            developer_fields.push(DeveloperFieldDefinition {
                field_number,
                field_size: size,
                developer_data_index,
            });
            curr_idx += 3;
        }
    }

    let record = DefinitionRecord {
        header,
        architecture,
        global_message_number,
        field_definitions: fields,
        developer_field_definitions: developer_fields,
    };
    (record, curr_idx)
}

#[test]
fn test_parse_definition_record() {
    let header = NormalDefinitionHeader {
        contains_extended_definitions: false,
        local_message_type: 1,
    };
    let data: Vec<u8> = vec![
        0x00,
        0x01, // architecture
        0x0A,
        0x0B, // global message number
        0x02, // num of fields
        0x01,
        0x01,
        BaseType::Uint8.into(), // field definition
        0x02,
        0x04,
        BaseType::Uint16.into(),
    ];
    let expected = DefinitionRecord {
        header: header.clone(),
        architecture: Endianness::BigEndian,
        global_message_number: 0x0A0B,
        field_definitions: vec![
            FieldDefinition {
                field_definition_number: 1,
                field_size: 1,
                base_type: get_base_type_info(BaseType::Uint8.into()),
            },
            FieldDefinition {
                field_definition_number: 2,
                field_size: 4,
                base_type: get_base_type_info(BaseType::Uint16.into()),
            },
        ],
        developer_field_definitions: vec![],
    };
    let (actual, new_idx) = parse_definition_record(&data, header, 0);
    assert_eq!(new_idx, data.len());
    assert_eq!(actual, expected)
}

#[test]
fn test_parse_definition_record_with_developer_fields() {
    let header = NormalDefinitionHeader {
        contains_extended_definitions: true,
        local_message_type: 1,
    };
    let data: Vec<u8> = vec![
        0x00,
        0x01, // architecture
        0x0A,
        0x0B, // global message number
        0x02, //num of fields
        0x01,
        0x01,
        BaseType::Uint8.into(), // field definition
        0x02,
        0x04,
        BaseType::Uint16.into(),
        0x02, // num of dev fields
        0x01,
        0x01,
        0x01,
        0x02,
        0x02,
        0x02,
    ];
    let expected = DefinitionRecord {
        header: header.clone(),
        architecture: Endianness::BigEndian,
        global_message_number: 0x0A0B,
        field_definitions: vec![
            FieldDefinition {
                field_definition_number: 1,
                field_size: 1,
                base_type: get_base_type_info(BaseType::Uint8.into()),
            },
            FieldDefinition {
                field_definition_number: 2,
                field_size: 4,
                base_type: get_base_type_info(BaseType::Uint16.into()),
            },
        ],
        developer_field_definitions: vec![
            DeveloperFieldDefinition {
                field_number: 1,
                field_size: 1,
                developer_data_index: 1,
            },
            DeveloperFieldDefinition {
                field_number: 2,
                field_size: 2,
                developer_data_index: 2,
            },
        ],
    };
    let (actual, new_idx) = parse_definition_record(&data, header, 0);
    assert_eq!(new_idx, data.len());
    assert_eq!(actual, expected)
}

#[test]
#[should_panic]
fn test_parse_definition_record_invalid_size() {
    let header = NormalDefinitionHeader {
        contains_extended_definitions: false,
        local_message_type: 1,
    };
    let data: Vec<u8> = vec![
        0x00, 0x01, // architecture
        0x0A, 0x0B, // global message number
        0x01, //num of fields
        0x01, 0x01, 0x09, // field definition
    ];
    parse_definition_record(&data, header, 0);
}

use codegen::Scope;
use std::error::Error;

use std::fs::File;

fn main() {
    // println!("cargo:rerun-if-changed=fit_definitions/profile_messages.csv");
    // println!("cargo:rerun-if-changed=fit_definitions/profile_types.csv");
    let mut codegen_scope = Scope::new();
    let profile_types = read_profile_types("fit_definitions/profile_types.csv");
    let messages = read_messages("fit-definitions/messages.csv");
    println!("profile_types : {:#?}", profile_types);
}

#[derive(Debug, Clone, PartialEq)]
struct FitMessage {
    name: String,
    comment: Option<String>,
    fields: Vec<FitMessageField>,
}

#[derive(Debug, Clone, PartialEq)]
struct FitMessageField {
    field_definition_number: u8,
    name: String,
    field_type: String,
    array_len: String,
    scale: f32,
    components: Vec<String>,
    units: Option<String>,
    bits: Vec<u8>,
    ref_field_name: Vec<String>,
    ref_field_value: Vec<String>,
    comment: Option<String>,
    example: Option<u8>,
}

enum FitMessageFields {
    MessageName = 0,
    FieldDefNumber = 1,
    FieldName = 2,
    FieldType = 3,
    Array = 4,
    Components = 5,
    Scale = 6,
    Offset = 7,
    Units = 8,
    Bits = 9,
    Accumulate = 10,
    RefFieldName = 11,
    RefFieldValue = 12,
    Comment = 13,
    Example = 14,
}
struct FitMsgRecFieldIndxs {}

fn read_messages(file_path: &str) -> Result<Vec<FitMessage>, Box<dyn Error>> {
    const MESSAGE_NAME_IDX: usize = 0;
    const FIELD_DEF_NUMBER_IDX: usize = 1;
    const FIELD_NAME_IDX: usize = 2;
    const FIELD_TYPE_IDX: usize = 3;
    const ARRAY_IDX: usize = 4;
    const COMPONENTS_IDX: usize = 5;
    const SCALE_IDX: usize = 6;
    const OFFSET_IDX: usize = 7;
    const UNITS_IDX: usize = 8;
    const BITS_IDX: usize = 9;
    const ACCUMULATE_IDX: usize = 10;
    const REF_FIELD_NAME_IDX: usize = 11;
    const REF_FIELD_VALUE_IDX: usize = 12;
    const COMMENT_IDX: usize = 13;
    const EXAMPLE_IDX: usize = 14;

    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut fit_messages: Vec<FitMessage> = Vec::new();
    let records = rdr.records();
    let mut first_message = true;
    let mut curr_message = FitMessage {
        name: String::new(),
        comment: None,
        fields: Vec::new(),
    };
    for res in records {
        let rec = res?;
        if matches!(rec.get(MESSAGE_NAME_IDX), Some(mn) if !mn.is_empty()) {
            if !first_message {
                fit_messages.push(curr_message);
                first_message = false;
            }
            curr_message = FitMessage {
                name: rec[MESSAGE_NAME_IDX].to_string(),
                comment: Some(rec[COMMENT_IDX].to_string()),
                fields: Vec::new(),
            };
            println!("Starting message for {}", curr_message.name);
        } else {
            // TODO: parse message fields
        }
    }
    fit_messages.push(curr_message);
    Ok(fit_messages)
}

fn read_profile_types(file_path: &str) -> Result<Vec<FitType>, Box<dyn Error>> {
    const TYPE_NAME_RECORD_IDX: usize = 0;
    const BASE_TYPE_RECORD_IDX: usize = 1;
    const VALUE_NAME_RECORD_IDX: usize = 2;
    const VALUE_RECORD_IDX: usize = 3;
    const COMMENT_RECORD_IDX: usize = 4;

    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut fit_types: Vec<FitType> = Vec::new();
    let mut curr_fit_type: FitType = FitType {
        type_name: String::new(),
        base_type: String::new(),
        values: vec![],
    };
    let mut first_type = true;
    let records = rdr.records();
    for res in records {
        let rec = res?;
        if matches!(rec.get(TYPE_NAME_RECORD_IDX), Some(tn) if !tn.is_empty()) {
            // starting new fit type definitions
            if !first_type {
                // not first item in CSV file
                fit_types.push(curr_fit_type);
                first_type = false;
            }

            curr_fit_type = FitType {
                type_name: rec[TYPE_NAME_RECORD_IDX].to_string(),
                base_type: rec[BASE_TYPE_RECORD_IDX].to_string(),
                values: Vec::new(),
            };
            println!("Starting value for {}", curr_fit_type.type_name);
        } else {
            // continuing to add fit type values to current fit type
            let value_name = &rec[VALUE_NAME_RECORD_IDX];
            let value_str = &rec[VALUE_RECORD_IDX].trim();
            let comment = &rec[COMMENT_RECORD_IDX];
            println!(
                "Adding to type {} a value name {} with value {}",
                curr_fit_type.type_name, value_name, value_str
            );
            let value = if value_str.to_lowercase().starts_with("0x") {
                u32::from_str_radix(value_str.to_lowercase().trim_start_matches("0x"), 16)?
            } else {
                value_str.parse::<u32>()?
            };
            curr_fit_type.values.push(FitTypeValue {
                value_name: value_name.to_string(),
                value,
                comment: comment.to_string(),
            })
        }
    }
    fit_types.push(curr_fit_type);
    Ok(fit_types)
}

#[derive(Debug, Clone, PartialEq)]
struct FitTypeValue {
    value_name: String,
    value: u32,
    comment: String,
}

#[derive(Debug, Clone, PartialEq)]
struct FitType {
    type_name: String,
    base_type: String,
    values: Vec<FitTypeValue>,
}

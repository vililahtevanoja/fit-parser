use std::error::Error;

use std::fs::File;
use std::str::FromStr;

use convert_case::{Case, Casing};

#[derive(Debug, Clone, PartialEq)]
pub struct FitMessage {
    pub name: String,
    pub comment: Option<String>,
    pub fields: Vec<FitMessageField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FitRefField {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FitMessageField {
    pub category: String,
    pub definition_number: u8,
    pub name: String,
    pub field_type: String,
    pub array: FitMessageArrayType,
    pub scale: Vec<f32>,
    pub offset: i16,
    pub components: Vec<String>,
    pub units: Vec<String>,
    pub bits: Vec<u8>,
    pub accumulate: Vec<u8>,
    pub ref_fields: Vec<FitRefField>,
    pub comment: Option<String>,
    pub example: Option<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FitMessageArrayType {
    NotArray,
    FixedSizeArray(usize),
    VariableSizeArray,
}

pub fn read_messages(file_path: &str) -> Result<Vec<FitMessage>, Box<dyn Error>> {
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
    const PRODUCTS_IDX: usize = 14;
    const EXAMPLE_IDX: usize = 15;

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
    let mut curr_category = String::new();
    for res in records {
        let rec = res?.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        assert_eq!(rec.len(), EXAMPLE_IDX + 1);
        if rec[MESSAGE_NAME_IDX].is_empty()
            && rec[FIELD_DEF_NUMBER_IDX].is_empty()
            && rec[FIELD_NAME_IDX].is_empty()
            && !rec[FIELD_TYPE_IDX].is_empty()
        {
            curr_category = rec[FIELD_TYPE_IDX].to_string();
            println!("Starting category {}", curr_category)
        }
        if matches!(rec.get(MESSAGE_NAME_IDX), Some(mn) if !mn.is_empty()) {
            if first_message {
                fit_messages.push(curr_message.clone());
                first_message = false;
            }
            fit_messages.push(curr_message.clone());
            curr_message = FitMessage {
                name: rec[MESSAGE_NAME_IDX].to_string(),
                comment: Some(rec[COMMENT_IDX].to_string()).filter(|s| !s.is_empty()),
                fields: Vec::new(),
            };
            println!("Starting message for {}", curr_message.name);
        } else {
            let field_def_number_str = rec[FIELD_DEF_NUMBER_IDX].to_string();
            if field_def_number_str.is_empty() {
                continue; // let's not handle fields such as product subfields favero_product and garmin_product yet
            }
            let definition_number = field_def_number_str.parse::<u8>()?;
            let name = rec[FIELD_NAME_IDX].clone();
            let field_type = rec[FIELD_TYPE_IDX].clone();
            let array = parse_fit_message_array(&rec[ARRAY_IDX]);
            let components = parse_comma_delimited_string_list(&rec[COMPONENTS_IDX]);
            let scale = parse_fit_message_scale_record(&rec[SCALE_IDX]);
            let default_scale_used = rec[SCALE_IDX].is_empty();
            let offset = parse_fit_message_offset_record(&rec[OFFSET_IDX]);
            let units = parse_comma_delimited_string_list(&rec[UNITS_IDX]);
            let bits = parse_fit_message_bits(&rec[BITS_IDX]);
            let accumulate = parse_fit_message_accumulate(&rec[ACCUMULATE_IDX]);
            validate_components_with_scale(
                &curr_message.name,
                &name,
                &components,
                &scale,
                default_scale_used,
            );
            validate_components_with_bits(&curr_message.name, &name, &components, &bits);
            validate_components_with_accumulate(
                &curr_message.name,
                &name,
                &components,
                &accumulate,
            );
            let ref_field_names = parse_comma_delimited_string_list(&rec[REF_FIELD_NAME_IDX]);
            let ref_field_values = parse_comma_delimited_string_list(&rec[REF_FIELD_VALUE_IDX]);
            validate_ref_field_names_and_values(
                &curr_message.name,
                &name,
                &ref_field_names,
                &ref_field_values,
            );
            assert_eq!(
                ref_field_names.len(),
                ref_field_values.len(),
                "Ref Field Value count should match Ref Field Names. {:#?} vs {:#?}",
                ref_field_names,
                ref_field_values
            );
            let ref_fields = ref_field_names
                .iter()
                .zip(ref_field_values.iter())
                .map(|(name, value)| FitRefField {
                    name: name.to_string(),
                    value: value.to_string(),
                })
                .collect::<Vec<FitRefField>>();
            let comment = Some(rec[COMMENT_IDX].clone()).filter(|s| !s.is_empty());
            let _products = rec[PRODUCTS_IDX].clone(); //not used?
            let example = Some(&rec[EXAMPLE_IDX])
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<u8>().unwrap());
            let fit_msg_field = FitMessageField {
                category: curr_category.clone(),
                definition_number,
                name,
                field_type,
                array,
                components,
                scale,
                offset,
                units,
                bits,
                accumulate,
                ref_fields,
                comment,
                example,
            };
            curr_message.fields.push(fit_msg_field);
        }
    }
    fit_messages.push(curr_message);
    Ok(fit_messages)
}

fn validate_components_with_scale(
    message_name: &str,
    field_name: &str,
    components: &[String],
    scale: &[f32],
    default_scale_used: bool,
) {
    assert!(
        components.len() == scale.len()
            || default_scale_used
            || (components.is_empty() && scale.len() == 1),
        "Components and non-default scale len definition do not match for {}.{}: {:#?} -- {:#?}",
        message_name,
        field_name,
        components,
        scale
    );
}

fn validate_components_with_bits(
    message_name: &str,
    field_name: &str,
    components: &[String],
    bits: &[u8],
) {
    assert!(
        components.len() == bits.len() || (components.is_empty() && bits.len() == 1),
        "Components and bits len do not match for {}.{}: {:#?} -- {:#?}",
        message_name,
        field_name,
        components,
        bits
    );
}

fn validate_components_with_accumulate(
    message_name: &str,
    field_name: &str,
    components: &[String],
    accumulate: &[u8],
) {
    assert!(
        components.len() == accumulate.len() || accumulate.is_empty(),
        "Components and bits len do not match for {}.{}: {:#?} -- {:#?}",
        message_name,
        field_name,
        components,
        accumulate
    );
}

fn validate_ref_field_names_and_values(
    message_name: &str,
    field_name: &str,
    ref_field_names: &[String],
    ref_field_values: &[String],
) {
    assert_eq!(
        ref_field_names.len(),
        ref_field_values.len(),
        "Ref Field Name count and Ref Field value count mismatch for {}.{}: {:#?} -- {:#?}",
        message_name,
        field_name,
        ref_field_names,
        ref_field_values
    )
}

fn parse_from_string_or_panic<T: FromStr>(s: &str, msg: &str) -> T {
    s.parse::<T>().unwrap_or_else(|_| panic!("{}", msg))
}

fn parse_fit_message_scale_record(s: &str) -> Vec<f32> {
    if s.is_empty() {
        vec![1.0]
    } else {
        s.split(',')
            .map(|s| {
                s.trim()
                    .parse::<f32>()
                    .unwrap_or_else(|_| panic!("Could not parse f32 scale from value {}", s))
            })
            .collect::<Vec<f32>>()
    }
}

fn parse_fit_message_offset_record(s: &str) -> i16 {
    if s.is_empty() {
        0
    } else {
        s.parse::<i16>()
            .unwrap_or_else(|_| panic!("Could not parse offset from value {}", s))
    }
}

fn parse_fit_message_bits(s: &str) -> Vec<u8> {
    if s.is_empty() {
        Vec::new()
    } else {
        s.split(',')
            .map(|s| {
                s.trim()
                    .parse::<u8>()
                    .unwrap_or_else(|_| panic!("Could not parse u8 bits value from value {}", s))
            })
            .collect::<Vec<u8>>()
    }
}

fn parse_fit_message_accumulate(input: &str) -> Vec<u8> {
    if input.is_empty() {
        Vec::new()
    } else {
        input
            .split(',')
            .map(|s| {
                s.trim().parse::<u8>().unwrap_or_else(|_| {
                    panic!("Could not parse accumulate u8 values from value {}", input)
                })
            })
            .collect::<Vec<u8>>()
    }
}

fn parse_comma_delimited_string_list(input: &str) -> Vec<String> {
    if input.is_empty() {
        Vec::new()
    } else {
        input
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    }
}

// Parses a FIT message array type from string representation
// `` => not an array
// `[3] => fixed size array
// `[N]` => variable length array
fn parse_fit_message_array(array_def: &str) -> FitMessageArrayType {
    match array_def.trim() {
        s if s.is_empty() => FitMessageArrayType::NotArray,
        s if s.eq_ignore_ascii_case("[N]") => FitMessageArrayType::VariableSizeArray,
        s if s.starts_with('[') && s.ends_with(']') => {
            let trimmed = &s[1..s.len() - 1];
            let parsed = trimmed.parse::<usize>();
            let array_size =
                parsed.unwrap_or_else(|_| panic!("Could not parse usize from value {}", s));
            FitMessageArrayType::FixedSizeArray(array_size)
        }
        weird_value => panic!("Weird fit message array value: {}", weird_value),
    }
}

#[test]
fn test_parse_empty_fit_message_array_definition() {
    let res = parse_fit_message_array("");
    assert_eq!(res, FitMessageArrayType::NotArray)
}

#[test]
fn test_parse_fixed_size_fit_message_array_definition() {
    let res = parse_fit_message_array("[3]");
    assert_eq!(res, FitMessageArrayType::FixedSizeArray(3))
}

#[test]
fn test_parse_variable_size_fit_message_arra_definition() {
    let res = parse_fit_message_array("[N]");
    assert_eq!(res, FitMessageArrayType::VariableSizeArray);
}

#[derive(Debug, Clone, PartialEq)]
pub struct FitTypeValue {
    pub value_name: String,
    pub value: u32,
    pub comment: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FitType {
    pub type_name: String,
    pub base_type: String,
    pub values: Vec<FitTypeValue>,
}

pub fn read_profile_types(file_path: &str) -> Result<Vec<FitType>, Box<dyn Error>> {
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
                fit_types.push(curr_fit_type.clone());
                first_type = false;
            }
            fit_types.push(curr_fit_type.clone());
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

fn fit_type_to_rust_type(fit_type: &str) -> &str {
    match fit_type {
        "uint8" => "u8",
        "uint8z" => "u8",
        "uint16" => "u16",
        "uint16z" => "u16",
        "uint32" => "u32",
        "uint32z" => "u32",
        _ => panic!("Unknown FIT type: {}", fit_type),
    }
}

pub fn generate_enum_type_as_string(t: FitType) -> String {
    let mut s = String::new();
    if t.base_type != "enum" {
        return s;
    }
    s.push_str(&format!(
        "pub enum {} {{\n",
        t.type_name.to_case(Case::UpperCamel)
    ));
    for val in t.values {
        if val.comment.trim().to_lowercase().starts_with("deprecated") {
            continue;
        }
        let comment = if val.comment.is_empty() {
            String::new()
        } else {
            format!(" // {}", val.comment)
        };
        s.push_str(&format!(
            "    {} = {},{}\n",
            val.value_name.to_case(Case::UpperCamel),
            val.value,
            comment
        ));
    }
    s.push_str("}\n");
    s
}

pub fn generate_fit_trait_as_string(t: FitType) -> String {
    let mut s = String::new();
    let type_name_cased = t.type_name.to_case(Case::UpperCamel);
    s.push_str(&format!("trait {}Trait {{\n", type_name_cased));
    let rust_type = fit_type_to_rust_type(&t.base_type);
    for val in t.values {
        let comment = if val.comment.is_empty() {
            String::new()
        } else {
            format!(" // {}", val.comment)
        };
        let value_name_cased = if val.value_name.chars().next().unwrap().is_digit(10) {
            format!("_{}", val.value_name.to_case(Case::UpperSnake))
        } else {
            val.value_name.to_case(Case::UpperSnake)
        };

        s.push_str(&format!(
            "    const {}: {} = {};{}\n",
            value_name_cased, rust_type, val.value, comment
        ))
    }
    s.push_str("}\n");
    s.push_str(&format!("struct {};\n", type_name_cased));
    s.push_str(&format!(
        "impl {}Trait for {}{{}}\n\n",
        type_name_cased, type_name_cased
    ));
    s
}

use std::error::Error;
use std::io::Read;

use codegen::{Scope, Variant};
use convert_case::Case;
use convert_case::Casing;
use fit_profile_typegen::{
    generate_enum_type_as_string, generate_fit_trait_as_string, read_messages, read_profile_types,
};

fn main() -> Result<(), Box<dyn Error>> {
    // println!("cargo:rerun-if-changed=fit_definitions/profile_messages.csv");
    // println!("cargo:rerun-if-changed=fit_definitions/profile_types.csv");
    let mut codegen_scope = Scope::new();
    let mut profiles_csv_content = String::new();
    let mut messages_csv_content = String::new();
    std::fs::File::open("../fit_definitions/profile_types.csv")
        .unwrap()
        .read_to_string(&mut profiles_csv_content)?;

    std::fs::File::open("../fit_definitions/profile_messages.csv")
        .unwrap()
        .read_to_string(&mut messages_csv_content)?;
    let types = read_profile_types(profiles_csv_content)?;
    let messages = read_messages(messages_csv_content)?;
    println!("profile_types : {:#?}", types);
    println!("messages: {:#?}", messages);
    let mut test_enum = codegen_scope.new_enum(&"test_enum".to_case(Case::UpperCamel));
    let mut variant = Variant::new(&"test_variant".to_case(Case::UpperCamel));
    variant.tuple("i64");
    test_enum.push_variant(variant);
    for t in types {
        if t.type_name.is_empty() {
            continue;
        } else if t.base_type == "enum" {
            println!("{}", generate_enum_type_as_string(t))
        } else {
            println!("{}", generate_fit_trait_as_string(t))
        }
    }
    //println!("{}\n", codegen_scope.to_string());
    Ok(())
}

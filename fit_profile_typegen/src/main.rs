use std::collections::HashMap;

use codegen::Field;
use codegen::{Scope, Variant};
use convert_case::Case;
use convert_case::Casing;
use fit_profile_typegen::FitType;
use fit_profile_typegen::{
    generate_enum_type_as_string, generate_fit_trait_as_string, read_messages, read_profile_types,
};

fn main() {
    // println!("cargo:rerun-if-changed=fit_definitions/profile_messages.csv");
    // println!("cargo:rerun-if-changed=fit_definitions/profile_types.csv");
    let mut codegen_scope = Scope::new();
    let types = read_profile_types("../fit_definitions/profile_types.csv").unwrap();
    let messages = read_messages("../fit_definitions/profile_messages.csv").unwrap();
    //println!("profile_types : {:#?}", profile_types);
    //println!("messages: {:#?}", messages)
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
}

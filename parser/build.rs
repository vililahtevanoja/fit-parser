use codegen::{Scope, Variant};

extern crate fit_profile_typegen;
use convert_case::{Case, Casing};
use fit_profile_typegen::generate_enum_type_as_string;
use fit_profile_typegen::generate_fit_trait_as_string;
use fit_profile_typegen::read_messages;
use fit_profile_typegen::read_profile_types;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;

fn main() -> Result<(), Box<dyn Error>> {
    // println!("cargo:rerun-if-changed=fit_definitions/profile_messages.csv");
    // println!("cargo:rerun-if-changed=fit_definitions/profile_types.csv");
    let out_dir = env::var("OUT_DIR").unwrap();
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
    println!("profile_types: {:#?}", types);
    println!("messages: {:#?}", messages);

    let mut codegen_str = String::new();
    for t in types {
        if t.type_name.is_empty() {
            continue;
        } else if t.base_type == "enum" {
            codegen_str.push_str(&generate_enum_type_as_string(t));
        } else {
            codegen_str.push_str(&generate_fit_trait_as_string(t));
        }
        codegen_str.push('\n')
    }
    println!("Writing output to {}/fit.rs", out_dir);
    let mut f = File::create(format!("{}/fit.rs", out_dir))?;
    f.write_all(codegen_str.as_bytes())?;
    f.sync_all()?;
    Ok(())
}

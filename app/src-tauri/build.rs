use std::{fs::{self, remove_dir_all, File}, io::Write, path::{Path, PathBuf}};
use std::io::{BufRead, BufReader};
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri_build::build();

    let mut config = prost_build::Config::new();
    config.message_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.enum_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    config.message_attribute(".", "#[serde(rename_all = \"camelCase\")]");
    config.enum_attribute(".", "#[serde(rename_all = \"camelCase\")]");

    config.protoc_arg("--proto_path=proto");

    let out_dir = Path::new("src/proto");
    if out_dir.exists() {
        //  remove file first
        remove_dir_all(out_dir)?;
    }

    fs::create_dir(out_dir)?;
    
    config.out_dir(out_dir);

    config.compile_protos(&["proto/k8s.io/api/core/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/apps/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/rbac/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/storage/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/discovery/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/discovery/v1beta1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/flowcontrol/v1/generated.proto"], &["src"])?;
    // config.compile_protos(&["proto/k8s.io/api/flowcontrol/v1beta1/generated.proto"], &["src"])?;
    // config.compile_protos(&["proto/k8s.io/api/flowcontrol/v1beta2/generated.proto"], &["src"])?;
    // config.compile_protos(&["proto/k8s.io/api/flowcontrol/v1beta3/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/coordination/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/scheduling/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/batch/v1/generated.proto"], &["src"])?;
    // config.compile_protos(&["proto/k8s.io/api/batch/v1beta1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/networking/v1/generated.proto"], &["src"])?;
    // config.compile_protos(&["proto/k8s.io/api/networking/v1beta1/generated.proto"], &["src"])?;
    // config.compile_protos(&["proto/k8s.io/api/networking/v1alpha1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/admissionregistration/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/admissionregistration/v1beta1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/admissionregistration/v1alpha1/generated.proto"], &["src"])?;

    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/apis/meta/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/apis/meta/v1beta1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/runtime/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/runtime/schema/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/util/intstr/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/api/resource/generated.proto"], &["src"])?;

    let mut file_vec = vec![];
    for entry in fs::read_dir(out_dir)? {
        let path = entry?.path();
        let file_name = path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

        let prefix = &file_name[0..file_name.len() - 3];

        file_vec.push(String::from(prefix));
    }

    for file in file_vec {
        let mut to = PathBuf::from(out_dir);

        for ele in file.split(".") {
            to = to.join(ele);
        }

        fs::create_dir_all(&to)?;
        to = to.join("mod.rs");

        let from = PathBuf::from(out_dir)
            .join(format!("{}.rs", file));

        modify_file(from, to)?;
    }

    add_mod(out_dir.into())?;
    
    Ok(())
}

fn modify_file(from: PathBuf, to: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut to_file = File::create_new(&to)?;
    let from_file = File::open(&from)?;
    let reader = BufReader::new(from_file);

    let option_re = Regex::new(r"^(\s*)pub [a-zA-Z0-9_#]+: ::core::option::Option<.*$")?;
    let vec_re = Regex::new(r"^(\s*)pub [a-zA-Z0-9_#]+: ::prost::alloc::vec::Vec<.*$")?;
    for line in reader.lines() {
        let line = line?;
        if let Some(captures) = option_re.captures_iter(line.as_str()).next() {
            if let Some(prefix) = captures.get(1) {
                let attribute = format!("{}#[serde(skip_serializing_if = \"::core::option::Option::is_none\")]\n", prefix.as_str());
                to_file.write(attribute.as_bytes())?;
            }
        }

        if let Some(captures) = vec_re.captures_iter(line.as_str()).next() {
            if let Some(prefix) = captures.get(1) {
                let attribute = format!("{}#[serde(skip_serializing_if = \"::prost::alloc::vec::Vec::is_empty\")]\n", prefix.as_str());
                to_file.write(attribute.as_bytes())?;
            }
        }

        to_file.write_all(line.as_bytes())?;
        to_file.write(b"\n")?;
        to_file.flush()?;
    }

    fs::remove_file(from)?;
    Ok(())
}

fn add_mod(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut mod_content = String::new();
    for entry in fs::read_dir(&path)? {
        let en_path = entry?.path();
        if en_path.is_dir() {
            let file_name = en_path.file_name().unwrap().to_str().unwrap();

            mod_content.push_str(format!("pub mod {};\n", file_name).as_str());

            add_mod(en_path)?;
        }
    }

    if !mod_content.is_empty() {
        
        let mut mod_file = File::create_new(path.join("mod.rs"))?;
        mod_file.write(mod_content.as_bytes())?;
        mod_file.flush()?;
    }

    Ok(())
}
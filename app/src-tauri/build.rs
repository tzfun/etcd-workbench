use std::{fs::{self, remove_dir_all, File}, io::Write, path::{Path, PathBuf}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri_build::build();

    let mut config = prost_build::Config::new();
    config.message_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.enum_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    config.message_attribute(".", "#[serde(rename_all = \"camelCase\")]");
    config.enum_attribute(".", "#[serde(rename_all = \"camelCase\")]");
    

    config.protoc_arg("--proto_path=proto");
    // config.extern_path(".google.protobuf", "::pbjson_types");

    let out_dir = Path::new("src/proto");
    if out_dir.exists() {
        //  remove file first
        remove_dir_all(out_dir)?;
    }

    fs::create_dir(out_dir)?;
    
    config.out_dir(out_dir);


    config.compile_protos(&["proto/k8s.io/api/apps/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/core/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/rbac/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/storage/v1/generated.proto"], &["src"])?;

    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/apis/meta/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/runtime/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/runtime/schema/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/util/intstr/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/api/resource/generated.proto"], &["src"])?;

    let mut mod_content = String::new();
    for entry in fs::read_dir(out_dir)? {
        let path = entry?.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let prefix = &file_name[0..file_name.len() - 3];
        let mod_name = prefix.replace(".", "_");

        let rename = format!("{}.rs", mod_name);
        let rename_file = PathBuf::from(out_dir).join(rename.clone());
        fs::rename(path, rename_file)?;

        mod_content.push_str("pub mod ");
        mod_content.push_str(mod_name.as_str());
        mod_content.push_str(";\n");
    }
    
    let mut mod_file = File::create_new(PathBuf::from(out_dir).join("mod.rs"))?;
    mod_file.write(mod_content.as_bytes())?;
    mod_file.flush()?;
    Ok(())
}

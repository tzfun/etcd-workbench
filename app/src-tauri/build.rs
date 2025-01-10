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

        println!("mkdirs {:?}", to);
        fs::create_dir_all(&to)?;
        to = to.join("mod.rs");

        let from = PathBuf::from(out_dir)
            .join(format!("{}.rs", file));

        File::create_new(&to)?;
        fs::copy(from.clone(), to)?;

        fs::remove_file(from)?;
    }

    add_mod(out_dir.into())?;
    
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
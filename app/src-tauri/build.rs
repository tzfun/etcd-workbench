fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri_build::build();

    let mut config = prost_build::Config::new();
    config.message_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.enum_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    config.message_attribute(".", "#[serde(rename_all = \"camelCase\")]");
    config.enum_attribute(".", "#[serde(rename_all = \"camelCase\")]");
    

    config.protoc_arg("--proto_path=proto");
    // config.extern_path(".google.protobuf", "::pbjson_types");

    config.compile_protos(&["proto/k8s.io/api/apps/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/api/core/v1/generated.proto"], &["src"])?;

    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/apis/meta/v1/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/runtime/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/runtime/schema/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/util/intstr/generated.proto"], &["src"])?;
    config.compile_protos(&["proto/k8s.io/apimachinery/pkg/api/resource/generated.proto"], &["src"])?;

    Ok(())
}

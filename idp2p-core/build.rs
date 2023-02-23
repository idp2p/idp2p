fn main() {
    let mut prost_build = prost_build::Config::new();
    prost_build.protoc_arg("--experimental_allow_proto3_optional");

    prost_build
        .compile_protos(&["proto/id.proto", "proto/message.proto"], &["proto"])
        .unwrap();
}
fn main() {
    let mut prost_build = prost_build::Config::new();
    prost_build
        .compile_protos(&["proto/id.proto"], &["proto"])
        .unwrap();
}
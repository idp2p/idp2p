fn main() {
    prost_build::compile_protos(
        &[
            "proto/identity.proto",
            "proto/message.proto",
        ],
        &["proto"],
    )
    .unwrap();
}

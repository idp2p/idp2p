fn main() {
    prost_build::compile_protos(
        &[
            "proto/common.proto",
            "proto/identity.proto",
            "proto/message.proto",
        ],
        &["proto"],
    )
    .unwrap();
}

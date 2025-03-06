fn main() {
    println!("cargo:rerun-if-changed=../proto/message.proto");
    println!("cargo:rerun-if-changed=../proto/protocol.proto");

    prost_build::compile_protos(&[
        "../proto/message.proto",
        "../proto/protocol.proto",
    ], &["../proto/"]).unwrap();
}

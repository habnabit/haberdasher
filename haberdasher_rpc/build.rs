fn main() {
    prost_build::compile_protos(
        &["src/haberdasher.proto"],
        &["src/"],
    ).unwrap();
}

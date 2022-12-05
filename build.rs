use std::{env, path::PathBuf};

fn main() {
    tonic_build::compile_protos("protos/master.proto");
    tonic_build::compile_protos("protos/worker.proto");

    // let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // tonic_build::configure()
    //     .build_client(true)
    //     .out_dir("proto")
    //     .compile(&["protos/master"], &["proto"])
    //     .expect("failed to compile protos");

    // Ok(())
}
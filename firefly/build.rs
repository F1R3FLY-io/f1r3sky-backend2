use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(
            &[
                "protobuf/DeployServiceV1.proto",
                "protobuf/ProposeServiceV1.proto",
            ],
            &["protobuf/", "protobuf/protobuf_external"],
        )
}

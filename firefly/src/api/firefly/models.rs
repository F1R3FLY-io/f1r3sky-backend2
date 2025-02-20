#![allow(clippy::large_enum_variant, clippy::enum_variant_names)]

pub mod servicemodelapi {
    tonic::include_proto!("servicemodelapi");
}

pub mod rhoapi {
    tonic::include_proto!("rhoapi");
}

pub mod casper {
    tonic::include_proto!("casper");

    pub mod v1 {
        tonic::include_proto!("casper.v1");
    }
}

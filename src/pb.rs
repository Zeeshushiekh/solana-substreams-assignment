// Equivalent to prost-generated definitions from proto/transfers.proto.
// Checked into source so a separate protoc installation is not required for Rust builds.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transfer {
    #[prost(string, tag = "1")]
    pub from: String,
    #[prost(string, tag = "2")]
    pub to: String,
    #[prost(uint64, tag = "3")]
    pub amount: u64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transfers {
    #[prost(message, repeated, tag = "1")]
    pub transfers: Vec<Transfer>,
}

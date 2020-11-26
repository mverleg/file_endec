pub use errors::FedResult;

#[allow(clippy::module_inception)]
pub mod base;
pub mod errors;
pub mod pth;
pub mod test_cmd;
pub mod version;
pub mod option;
pub mod rounding;

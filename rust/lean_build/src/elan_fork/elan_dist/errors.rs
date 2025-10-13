#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid toolchain name: '{0}'")]
    InvalidToolchainName(String),
}

pub type Result<T> = std::result::Result<T, Error>;

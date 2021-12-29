#[derive(Debug)]
pub enum DataError {
    IOError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
    Ron(ron::Error),
    Bincode(bincode::Error),
    #[cfg(feature = "io")]
    File(engine::error::FileError),
    // #[cfg(target_arch = "wasm32")]
    #[cfg(all(feature = "io", target_arch = "wasm32"))]
    QuadStorageError,
    #[cfg(all(feature = "io", target_arch = "wasm32"))]
    Base64(base64::DecodeError),
}

impl std::error::Error for DataError {}

impl core::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl From<std::io::Error> for DataError {
    fn from(error: std::io::Error) -> Self {
        Self::IOError(error)
    }
}

impl From<std::str::Utf8Error> for DataError {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::Utf8Error(error)
    }
}

impl From<ron::Error> for DataError {
    fn from(error: ron::Error) -> Self {
        Self::Ron(error)
    }
}

impl From<bincode::Error> for DataError {
    fn from(error: bincode::Error) -> Self {
        Self::Bincode(error)
    }
}

#[cfg(feature = "io")]
impl From<engine::error::FileError> for DataError {
    fn from(error: engine::error::FileError) -> Self {
        Self::File(error)
    }
}

#[cfg(all(feature = "io", target_arch = "wasm32"))]
impl From<base64::DecodeError> for DataError {
    fn from(err: base64::DecodeError) -> Self {
        Self::Base64(err)
    }
}

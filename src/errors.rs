use rodio::decoder::DecoderError;

#[derive(Debug)]
pub enum SongAddError {
    DecoderError(DecoderError),
    IoError(std::io::Error),
    RusqliteError(rusqlite::Error),
}

impl From<DecoderError> for SongAddError {
    fn from(err: DecoderError) -> Self {
        SongAddError::DecoderError(err)
    }
}

impl From<std::io::Error> for SongAddError {
    fn from(err: std::io::Error) -> Self {
        SongAddError::IoError(err)
    }
}

impl From<rusqlite::Error> for SongAddError {
    fn from(err: rusqlite::Error) -> Self {
        SongAddError::RusqliteError(err)
    }
}
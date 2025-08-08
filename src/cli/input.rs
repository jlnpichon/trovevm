use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub enum InputSource {
    File(PathBuf),
    Stdin,
    String(String),
}

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("Can't read file: {0}")]
    File(String),

    #[error("Read error from stdin")]
    Stdin,
}

impl InputSource {
    pub fn read_to_string(&self) -> Result<String, InputError> {
        Ok(match self {
            InputSource::File(path) => std::fs::read_to_string(path)
                .map_err(|_| InputError::File(path.display().to_string()))?,
            InputSource::Stdin => {
                let mut buffer = String::new();
                io::stdin()
                    .read_to_string(&mut buffer)
                    .map_err(|_| InputError::Stdin)?;
                buffer
            }
            InputSource::String(s) => s.clone(),
        })
    }

    pub fn open_reader(&self) -> Result<Box<dyn BufRead + '_>, InputError> {
        match self {
            InputSource::File(path) => {
                let file =
                    File::open(path).map_err(|_| InputError::File(path.display().to_string()))?;
                Ok(Box::new(BufReader::new(file)))
            }
            InputSource::Stdin => Ok(Box::new(BufReader::new(io::stdin()))),
            InputSource::String(s) => Ok(Box::new(BufReader::new(s.as_bytes()))),
        }
    }

    pub fn source_name(&self) -> String {
        match self {
            InputSource::File(path_buf) => path_buf.display().to_string(),
            InputSource::Stdin => "<stdin>".to_string(),
            InputSource::String(_) => "<memory>".to_string(),
        }
    }
}

impl From<&str> for InputSource {
    fn from(s: &str) -> Self {
        if s == "-" {
            InputSource::Stdin
        } else {
            InputSource::File(s.into())
        }
    }
}

impl From<String> for InputSource {
    fn from(s: String) -> Self {
        InputSource::from(s.as_str())
    }
}

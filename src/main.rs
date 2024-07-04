use std::fs::File;
use std::io::{Read};
use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::Parser;
use thiserror::Error;

#[derive(Error, Debug)]
enum JpegError {
    #[error("Not a valid JPEG file")]
    InvalidJpeg,
    #[error("Unexpected EOF")]
    UnexpectedEof,
    #[error("SOF marker not found")]
    SofNotFound
}

struct JpegMetadata {
    width: u16,
    height: u16
}

impl JpegMetadata {
    fn from_file(path: &PathBuf) -> Result<Self> {
        let mut file = File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).with_context(|| format!("Failed to read file: {}", path.display()))?;
        Self::from_bytes(&buffer)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 2 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
            return Err(JpegError::InvalidJpeg.into());
        } 

        let mut metadata = JpegMetadata {
            width: 0,
            height: 0
        };

        let mut i = 2;
        while i < bytes.len() - 1 {
            if bytes[i] == 0xFF {
                let marker = bytes[i + 1];
                match marker {
                    0xC0 => {
                        // SOF0 marker - has dimensions
                        if i + 8 >= bytes.len() {
                            return Err(JpegError::UnexpectedEof.into());
                        }
                        metadata.height = u16::from_be_bytes([bytes[i+5], bytes[i+6]]);
                        metadata.width = u16::from_be_bytes([bytes[i+7], bytes[i+8]]);
                        return Ok(metadata);
                    }
                    _ => {
                        if i + 3 >= bytes.len() {
                            return Err(JpegError::UnexpectedEof.into());
                        }
                        let segment_length = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
                        i += segment_length + 2;
                    }
                }
            }
        }

        Err(JpegError::SofNotFound.into())
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let metadata = JpegMetadata::from_file(&args.file).context("Failed to decode JPEG")?;

    println!("Image dimensions: {}x{}", metadata.width, metadata.height);

    if args.verbose {
        println!("File path {}", args.file.display());
    }

    Ok(())
}


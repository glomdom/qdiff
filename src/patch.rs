use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use zstd::stream::{Decoder, Encoder};

#[derive(Debug)]
pub struct PatchFile {
    pub added: Vec<(usize, Vec<u8>)>, // (offset, data)
    pub removed: Vec<(usize, usize)>, // (offset, size)
}

impl PatchFile {
    pub fn new() -> Self {
        Self {
            added: Vec::new(),
            removed: Vec::new(),
        }
    }

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(Encoder::new(file, 3)?);

        writer.write_all(b"QDF1")?; // magic
        writer.write_all(&[1])?; // patch format version
        writer.write_all(&(self.added.len() as u32 + self.removed.len() as u32).to_le_bytes())?; // number of changes

        for &(offset, ref data) in &self.added {
            writer.write_all(&[0x01])?; // additions
            writer.write_all(&offset.to_le_bytes())?;
            writer.write_all(&(data.len() as u32).to_le_bytes())?;
            writer.write_all(data)?;
        }

        for &(offset, size) in &self.removed {
            writer.write_all(&[0x02])?; // removals
            writer.write_all(&offset.to_le_bytes())?;
            writer.write_all(&(size as u32).to_le_bytes())?;
        }

        writer.write_all(b"EOFQ")?;
        writer.flush()?;

        Ok(())
    }

    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(Decoder::new(file)?);

        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        if &magic != b"QDF1" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid patch format",
            ));
        }

        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;

        if version[0] != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported patch version",
            ));
        }

        let mut change_count = [0u8; 4];
        reader.read_exact(&mut change_count)?;
        let change_count = u32::from_le_bytes(change_count);

        let mut patch = PatchFile::new();

        for _ in 0..change_count {
            let mut change_type = [0u8; 1];
            reader.read_exact(&mut change_type)?;

            let mut offset_buf = [0u8; 8];
            reader.read_exact(&mut offset_buf)?;
            let offset = usize::from_le_bytes(offset_buf);

            let mut size_buf = [0u8; 4];
            reader.read_exact(&mut size_buf)?;
            let size = u32::from_le_bytes(size_buf) as usize;

            if change_type[0] == 0x01 {
                let mut data = vec![0; size];
                reader.read_exact(&mut data)?;
                patch.added.push((offset, data));
            } else if change_type[0] == 0x02 {
                patch.removed.push((offset, size));
            }
        }

        Ok(patch)
    }
}

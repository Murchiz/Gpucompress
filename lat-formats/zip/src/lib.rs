use lat_core::{Compressor, ArchiveEntry};
use std::io::{Read, Write, Cursor};
use zip::{ZipWriter, ZipArchive};
use zip::write::FileOptions;

pub struct ZipCompressor;

impl Compressor for ZipCompressor {
    fn compress(&self, entries: &[ArchiveEntry], _password: Option<&str>) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut buf));
            let options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            for entry in entries {
                writer.start_file(&entry.name, options).map_err(|e| e.to_string())?;
                writer.write_all(&entry.data).map_err(|e| e.to_string())?;
            }
            writer.finish().map_err(|e| e.to_string())?;
        }
        Ok(buf)
    }

    fn decompress(&self, archive_data: &[u8], _password: Option<&str>) -> Result<Vec<ArchiveEntry>, String> {
        let mut archive = ZipArchive::new(Cursor::new(archive_data)).map_err(|e| e.to_string())?;
        let mut entries = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            entries.push(ArchiveEntry {
                name: file.name().to_string(),
                data: buf,
            });
        }
        Ok(entries)
    }
}

use lat_core::{Compressor, ArchiveEntry};
use sevenz_rust::{SevenZWriter, SevenZArchive};
use std::io::Cursor;

pub struct SevenZCompressor;

impl Compressor for SevenZCompressor {
    fn compress(&self, entries: &[ArchiveEntry], _password: Option<&str>) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        let mut writer = SevenZWriter::new(Cursor::new(&mut buf)).map_err(|e| e.to_string())?;
        for entry in entries {
            writer.append_bytes(&entry.data, &entry.name).map_err(|e| e.to_string())?;
        }
        writer.finish().map_err(|e| e.to_string())?;
        Ok(buf)
    }

    fn decompress(&self, archive_data: &[u8], _password: Option<&str>) -> Result<Vec<ArchiveEntry>, String> {
        let mut archive = SevenZArchive::new(Cursor::new(archive_data)).map_err(|e| e.to_string())?;
        let mut entries = Vec::new();
        archive.for_each_file(|file, reader| {
            let mut buf = Vec::new();
            std::io::copy(reader, &mut buf).unwrap();
            entries.push(ArchiveEntry {
                name: file.name().to_string(),
                data: buf,
            });
            Ok(true)
        }).map_err(|e| e.to_string())?;
        Ok(entries)
    }
}

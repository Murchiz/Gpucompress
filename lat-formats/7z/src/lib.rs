use lat_core::{ArchiveEntry, Compressor};
use sevenz_rust::{SevenZArchiveEntry, SevenZReader, SevenZWriter};
use std::io::Cursor;

pub struct SevenZCompressor;

impl Compressor for SevenZCompressor {
    fn compress(
        &self,
        entries: &[ArchiveEntry],
        _password: Option<&str>,
    ) -> Result<Vec<u8>, String> {
        // Bolt ⚡ Optimization: Pre-allocate output buffer.
        // 7z compression is very effective, so uncompressed size is a safe upper bound.
        let total_uncompressed_size: usize = entries.iter().map(|e| e.data.len()).sum();
        let mut buf = Vec::with_capacity(total_uncompressed_size);

        let mut writer = SevenZWriter::new(Cursor::new(&mut buf)).map_err(|e| e.to_string())?;
        for entry in entries {
            let mut sz_entry = SevenZArchiveEntry::default();
            sz_entry.name = entry.name.clone();
            sz_entry.has_stream = true;
            sz_entry.size = entry.data.len() as u64;

            writer
                .push_archive_entry(sz_entry, Some(Cursor::new(&entry.data)))
                .map_err(|e| e.to_string())?;
        }
        writer.finish().map_err(|e| e.to_string())?;
        Ok(buf)
    }

    fn decompress(
        &self,
        archive_data: &[u8],
        _password: Option<&str>,
    ) -> Result<Vec<ArchiveEntry>, String> {
        let password = _password.map(|p| p.into()).unwrap_or_default();
        let mut reader = SevenZReader::new(
            Cursor::new(archive_data),
            archive_data.len() as u64,
            password,
        )
        .map_err(|e| e.to_string())?;

        // Bolt ⚡ Optimization: Pre-allocate the entries vector.
        let mut entries = Vec::with_capacity(reader.archive().files.len());

        reader
            .for_each_entries(|file, reader| {
                // Bolt ⚡ Optimization: Use read_exact into a pre-resized buffer instead of
                // std::io::copy to avoid redundant reallocations and EOF checks.
                let size = file.size() as usize;
                let mut buf = vec![0u8; size];
                std::io::Read::read_exact(reader, &mut buf)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

                entries.push(ArchiveEntry {
                    name: file.name().to_string(),
                    data: buf,
                });
                Ok(true)
            })
            .map_err(|e| e.to_string())?;

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lat_core::ArchiveEntry;

    #[test]
    fn test_7z_compress_decompress() {
        let compressor = SevenZCompressor;
        let entries = vec![
            ArchiveEntry {
                name: "test1.txt".to_string(),
                data: b"Hello 7z world".to_vec(),
            },
            ArchiveEntry {
                name: "folder/test2.txt".to_string(),
                data: b"More 7z data".to_vec(),
            },
        ];

        let compressed = compressor
            .compress(&entries, None)
            .expect("Compression failed");
        let decompressed = compressor
            .decompress(&compressed, None)
            .expect("Decompression failed");

        assert_eq!(entries.len(), decompressed.len());
        assert_eq!(entries[0].name, decompressed[0].name);
        assert_eq!(entries[0].data, decompressed[0].data);
        assert_eq!(entries[1].name, decompressed[1].name);
        assert_eq!(entries[1].data, decompressed[1].data);
    }
}

use lat_core::{ArchiveEntry, Compressor};
use std::io::{Cursor, Read, Write};
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

pub struct ZipCompressor;

impl Compressor for ZipCompressor {
    fn compress(
        &self,
        entries: &[ArchiveEntry],
        _password: Option<&str>,
    ) -> Result<Vec<u8>, String> {
        // Bolt ⚡ Optimization: Pre-allocate buffer with an accurate estimate of both
        // uncompressed data AND ZIP metadata overhead (headers, central directory).
        // This prevents multiple expensive reallocations for archives with many small files.
        // Use a single fold pass to calculate both totals efficiently.
        let (total_uncompressed_size, total_name_len) =
            entries.iter().fold((0, 0), |(size, name), entry| {
                (size + entry.data.len(), name + entry.name.len())
            });
        // Metadata overhead per file: 30 (Local File Header) + 46 (Central Directory Header) + 2 * name.len()
        // Moving the constant 76 bytes per entry and 22 bytes EOCD outside the loop reduces arithmetic operations.
        let total_overhead = 22 + (76 * entries.len()) + (2 * total_name_len);

        let mut buf = Vec::with_capacity(total_uncompressed_size + total_overhead);
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut buf));
            let options =
                FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            for entry in entries {
                writer
                    .start_file(&entry.name, options)
                    .map_err(|e| e.to_string())?;
                writer.write_all(&entry.data).map_err(|e| e.to_string())?;
            }
            writer.finish().map_err(|e| e.to_string())?;
        }
        Ok(buf)
    }

    fn decompress(
        &self,
        archive_data: &[u8],
        _password: Option<&str>,
    ) -> Result<Vec<ArchiveEntry>, String> {
        let mut archive = ZipArchive::new(Cursor::new(archive_data)).map_err(|e| e.to_string())?;

        // Pre-allocate the entries vector
        let mut entries = Vec::with_capacity(archive.len());

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;

            // Bolt ⚡ Optimization: Use read_exact into a pre-resized buffer instead of
            // read_to_end with capacity. This avoids redundant EOF checks and
            // additional read syscalls since the file size is already known.
            let mut buf = vec![0u8; file.size() as usize];
            file.read_exact(&mut buf).map_err(|e| e.to_string())?;

            entries.push(ArchiveEntry {
                name: file.name().to_string(),
                data: buf,
            });
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lat_core::ArchiveEntry;

    #[test]
    fn test_zip_compress_decompress() {
        let compressor = ZipCompressor;
        let entries = vec![
            ArchiveEntry {
                name: "test1.txt".to_string(),
                data: b"Hello world".to_vec(),
            },
            ArchiveEntry {
                name: "folder/test2.txt".to_string(),
                data: b"More data".to_vec(),
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

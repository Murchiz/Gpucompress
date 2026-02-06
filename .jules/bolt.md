## 2025-05-22 - [Optimized Zip Compression/Decompression]
**Learning:** Pre-allocating `Vec` capacity in ZIP operations (especially decompression) can yield massive speedups (up to 5x in benchmarks) by avoiding expensive reallocations and copies during the processing of many files.
**Action:** Always check for `Vec::with_capacity` opportunities when handling collections of known size, especially in loops involving I/O or decompression.

## 2025-05-23 - [Metadata Overhead in Capacity Estimation]
**Learning:** For archive formats like ZIP, pre-allocating only for uncompressed data is insufficient. Metadata overhead (headers, central directory) can be significant, especially for many small files, leading to frequent reallocations.
**Action:** Always include a calculated overhead for metadata when pre-allocating buffers for serialized formats. In ZIP, this is ~76 bytes + 2 * filename length per file, plus 22 bytes for the EOCD record.

## 2025-05-24 - [Crypto RNG and Allocation Optimization]
**Learning:** Initializing RNGs like `ThreadRng` multiple times in a single function call adds unnecessary overhead. Similarly, using high-level `decrypt` methods can lead to redundant copies of authentication tags when returning `Vec<u8>`. Early validation of input length in computationally expensive paths (like PBKDF2) is critical to prevent "denial of service" from malformed inputs.
**Action:** Reuse RNG instances within a scope. Use `decrypt_in_place_detached` when possible to control allocations. Always fail fast before starting expensive operations like PBKDF2.

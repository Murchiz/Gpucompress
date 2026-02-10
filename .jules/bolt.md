## 2025-05-22 - [Optimized Zip Compression/Decompression]
**Learning:** Pre-allocating `Vec` capacity in ZIP operations (especially decompression) can yield massive speedups (up to 5x in benchmarks) by avoiding expensive reallocations and copies during the processing of many files.
**Action:** Always check for `Vec::with_capacity` opportunities when handling collections of known size, especially in loops involving I/O or decompression.

## 2025-05-23 - [Metadata Overhead in Capacity Estimation]
**Learning:** For archive formats like ZIP, pre-allocating only for uncompressed data is insufficient. Metadata overhead (headers, central directory) can be significant, especially for many small files, leading to frequent reallocations.
**Action:** Always include a calculated overhead for metadata when pre-allocating buffers for serialized formats. In ZIP, this is ~76 bytes + 2 * filename length per file, plus 22 bytes for the EOCD record.

## 2025-05-24 - [Crypto RNG and Allocation Optimization]
**Learning:** Initializing RNGs like `ThreadRng` multiple times in a single function call adds unnecessary overhead. Similarly, using high-level `decrypt` methods can lead to redundant copies of authentication tags when returning `Vec<u8>`. Early validation of input length in computationally expensive paths (like PBKDF2) is critical to prevent "denial of service" from malformed inputs.
**Action:** Reuse RNG instances within a scope. Use `decrypt_in_place_detached` when possible to control allocations. Always fail fast before starting expensive operations like PBKDF2.

## 2025-05-25 - [Combined RNG Calls and Slicing Optimization]
**Learning:** Multiple small calls to random number generators or repetitive slicing can add unnecessary overhead in hot paths like crypto. Combining RNG fills for salt and nonce, and using `split_at` for buffer extraction, simplifies code and reduces micro-overhead.
**Action:** In cryptographic or low-latency code, batch RNG requests if possible. Use `split_at` or `split_at_mut` to partition buffers cleanly in one go.

## 2025-05-26 - [Optimized Read for Known Sizes]
**Learning:** Using `read_to_end` with a `Vec` that has capacity but no length still results in an extra `read` syscall to check for EOF. When the size is known exactly (like from a ZIP header), using `vec![0u8; size]` followed by `read_exact` is more efficient as it avoids the extra syscall and potential reallocations.
**Action:** Prefer `read_exact` into a pre-resized buffer when the total data size is known in advance.

## 2025-05-27 - [Direct Key Derivation and In-Place Decryption]
**Learning:** In cryptographic operations using `aes-gcm` and `pbkdf2`, deriving the key directly into the `Key<Aes256Gcm>` buffer avoids redundant copies and stack allocations. Using `decrypt_in_place` (from the `AeadInPlace` trait) on a contiguous buffer of `[ciphertext][tag]` simplifies code and allows the library to handle tag verification and buffer truncation automatically, reducing slicing overhead.
**Action:** Always aim to derive keys directly into their final container. Use `decrypt_in_place` when the archive format provides ciphertext and authentication tags contiguously.

## 2025-05-15 - Redundant allocations in crypto encryption
**Learning:** Using `cipher.encrypt` in the `aead` crate returns a newly allocated `Vec<u8>`, which often leads to redundant heap allocations and memory copies when the final result needs to include a salt and nonce.
**Action:** Use `encrypt_in_place_detached` from the `AeadInPlace` trait to encrypt data directly into a pre-allocated buffer that already contains the salt and nonce. This reduces allocations from 2 to 1 and copies from 2 to 1 for the main data payload, resulting in ~20% faster encryption for large files.

## 2025-05-22 - [Optimized Zip Compression/Decompression]
**Learning:** Pre-allocating `Vec` capacity in ZIP operations (especially decompression) can yield massive speedups (up to 5x in benchmarks) by avoiding expensive reallocations and copies during the processing of many files.
**Action:** Always check for `Vec::with_capacity` opportunities when handling collections of known size, especially in loops involving I/O or decompression.

## 2025-05-22 - [Optimized Zip Compression/Decompression]
**Learning:** Pre-allocating `Vec` capacity in ZIP operations (especially decompression) can yield massive speedups (up to 5x in benchmarks) by avoiding expensive reallocations and copies during the processing of many files.
**Action:** Always check for `Vec::with_capacity` opportunities when handling collections of known size, especially in loops involving I/O or decompression.

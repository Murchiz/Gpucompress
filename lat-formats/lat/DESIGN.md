# .lat Format Design

The `.lat` format is designed for maximum compression ratio using GPU parallelism. It avoids PAQ-style context mixing and neural networks, focusing instead on massively parallel match finding and high-order entropy coding.

## Core Components

1.  **Massive Window Matching**:
    *   Uses GPU-accelerated Suffix Array construction (e.g., using prefix doubling or SA-IS adapted for GPU).
    *   Supports window sizes up to several gigabytes, limited only by GPU VRAM.
    *   Finds longest prefixes across the entire window in parallel.

2.  **Optimal Parsing**:
    *   Implements a GPU-based forward-looking optimal parser.
    *   Calculates the bit-cost of various match/literal sequences and finds the near-optimal path using dynamic programming or a simplified heuristic suited for SIMT.

3.  **rANS (Range Asymmetric Numeral Systems)**:
    *   A high-performance entropy coder that allows for SIMD/GPU-friendly parallelization.
    *   Used for encoding literals, match lengths, and offsets.

4.  **Static and Dynamic Contexts**:
    *   Uses high-order contexts (e.g., order-4 literals) for probability estimation.
    *   Contexts are updated in blocks to maintain parallelism.

5.  **File Structure**:
    *   `[Header]` (Signature, Version, Flags)
    *   `[Metadata]` (Original filename, timestamps, permissions)
    *   `[Compressed Data Blocks]`
        *   `[Block Header]` (Uncompressed size, Compressed size, Coder state)
        *   `[Literals Stream]`
        *   `[Matches Stream]` (Offsets and Lengths)
    *   `[Encryption Footer]` (If enabled)

## GPU Acceleration Strategy

*   **CUDA (Windows)**: Implementation using custom kernels for Suffix Array and rANS.
*   **Vulkan (Android)**: Implementation using Compute Shaders (GLSL) for portability.

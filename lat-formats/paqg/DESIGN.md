# paqg (PAQ-GPU) Design

`paqg` is a GPU-accelerated port of PAQ-style compression. It focuses on accelerating the most computationally expensive parts of PAQ: context modeling and mixing.

## Core Components

1.  **Block-Parallel Context Modeling**:
    *   Divide the input into independent or overlapping blocks that can be modeled in parallel.
    *   Use shared memory on GPU to store and update context weights.

2.  **GPU-Accelerated Mixing**:
    *   The "Mixer" in PAQ combines probabilities from multiple models. This is highly parallelizable across bits if multiple streams are used.

3.  **Parallel Arithmetic Coding (PAC)**:
    *   PAQ traditionally uses bit-wise arithmetic coding, which is serial.
    *   `paqg` will use a multi-stream approach where the data is split into multiple independent streams, each with its own arithmetic coder, allowing the GPU to process thousands of streams in parallel.

4.  **Compatibility**:
    *   Aims for high ratio similar to PAQ8/ZPAQ but with orders of magnitude faster execution on GPU-equipped systems.

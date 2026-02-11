extern "C" __global__ void paq_mix_probabilities(
    const float* model_probs,
    const float* weights,
    float* output_probs,
    int num_models,
    int num_bits
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < num_bits) {
        float mixed_p = 0.0f;
        float total_w = 0.0f;
        // Bolt ⚡ Optimization: Using coalesced memory access pattern [num_models][num_bits].
        // This allows adjacent threads to access adjacent memory, significantly improving throughput.
        for (int i = 0; i < num_models; ++i) {
            float w = weights[i * num_bits + idx];
            mixed_p += w * model_probs[i * num_bits + idx];
            total_w += w;
        }
        output_probs[idx] = mixed_p / total_w;
    }
}

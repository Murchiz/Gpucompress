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
        for (int i = 0; i < num_models; ++i) {
            float w = weights[idx * num_models + i];
            mixed_p += w * model_probs[idx * num_models + i];
            total_w += w;
        }
        output_probs[idx] = mixed_p / total_w;
    }
}

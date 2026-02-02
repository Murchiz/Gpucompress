use lat_core::GpuAccelerator;
use wgpu;
use std::sync::Arc;

pub struct VulkanAccelerator {
    // wgpu abstracts over Vulkan/Metal/DX12
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl VulkanAccelerator {
    pub async fn new() -> Result<Self, String> {
        let instance = wgpu::Instance::default();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }).await.ok_or("Failed to find a GPU adapter")?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("VulkanAccelerator"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }, None).await.map_err(|e| e.to_string())?;

        Ok(Self { device, queue })
    }
}

impl GpuAccelerator for VulkanAccelerator {
    fn name(&self) -> &str {
        "Vulkan"
    }

    fn run_kernel(&self, name: &str, _data: &mut [u8]) -> Result<(), String> {
        println!("Running Vulkan (wgpu) compute shader: {}", name);
        Ok(())
    }

    fn mix_probabilities(&self, _model_probs: &[f32], _weights: &[f32], num_bits: usize) -> Result<Vec<f32>, String> {
        println!("Mixing probabilities on Vulkan for {} bits", num_bits);
        // Mock result
        Ok(vec![0.5; num_bits])
    }
}

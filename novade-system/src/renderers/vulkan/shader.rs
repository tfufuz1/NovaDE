use ash::{util::read_spv, vk, Device as AshDevice};
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

// ANCHOR: VulkanShaderModule Struct Definition
pub struct VulkanShaderModule {
    device: Arc<AshDevice>,
    shader_module: vk::ShaderModule,
}

// ANCHOR: VulkanShaderModule Implementation
impl VulkanShaderModule {
    // ANCHOR_EXT: new from SPIR-V code
    pub fn new_from_bytes(device: Arc<AshDevice>, spirv_code: &[u8]) -> Result<Self, String> {
        // The SPIR-V code is expected to be a slice of bytes, which ash::util::read_spv
        // can convert to a slice of u32.
        // Ensure the byte slice length is a multiple of 4.
        if spirv_code.len() % 4 != 0 {
            return Err(format!(
                "SPIR-V code length ({}) is not a multiple of 4.",
                spirv_code.len()
            ));
        }

        // The `read_spv` function expects an `std::io::Read` trait object.
        // We can use a `Cursor` to wrap the byte slice.
        // The code must be aligned to a 4-byte boundary.
        // `ash::util::read_spv` handles converting &[u8] to Vec<u32> if needed.
        // However, vk::ShaderModuleCreateInfo expects *const u32.
        // A common way is to ensure the input Vec<u8> is properly aligned and then cast its pointer.
        // Or, copy to Vec<u32>. Let's ensure our Vec<u8> becomes Vec<u32>.

        let mut cursor = Cursor::new(spirv_code);
        let code_u32 = read_spv(&mut cursor)
            .map_err(|e| format!("Failed to read SPIR-V bytes into u32 words: {}", e))?;

        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder()
            .code(&code_u32); // pCode expects a slice of u32

        let shader_module = unsafe {
            device
                .create_shader_module(&shader_module_create_info, None)
                .map_err(|e| format!("Failed to create shader module: {}", e))?
        };

        Ok(Self { device, shader_module })
    }

    // ANCHOR_EXT: new from file path
    pub fn new_from_file(device: Arc<AshDevice>, path_str: &str) -> Result<Self, String> {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("Shader file not found: {}", path_str));
        }

        let spirv_bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read shader file {}: {}", path_str, e))?;

        Self::new_from_bytes(device, &spirv_bytes)
    }

    // ANCHOR: Accessor for vk::ShaderModule
    pub fn handle(&self) -> vk::ShaderModule {
        self.shader_module
    }
}

// ANCHOR: VulkanShaderModule Drop Implementation
impl Drop for VulkanShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.shader_module, None);
        }
        // println!("VulkanShaderModule (handle: {:?}) dropped.", self.shader_module);
    }
}

// ANCHOR: Helper function to load SPIR-V (kept outside for general use if needed, or can be made private static method)
// This function is essentially duplicated by `new_from_file`'s internal logic now.
// It could be removed if `new_from_file` is the preferred public API for file loading.
// For now, keeping it as per original plan.
pub fn load_spirv_file(path_str: &str) -> Result<Vec<u8>, String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err(format!("Shader file not found: {}", path_str));
    }
    std::fs::read(path)
        .map_err(|e| format!("Failed to read shader file {}: {}", path_str, e))
}

use ash::{util::read_spv, vk, Device};
use std::io::Cursor;

/// Loads a SPIR-V shader module from a byte slice.
///
/// # Arguments
///
/// * `device`: The Vulkan logical device.
/// * `spirv_bytes`: A byte slice containing the SPIR-V code.
///
/// # Returns
///
/// A `Result` containing the created `vk::ShaderModule` or a `vk::Result` error.
///
/// # Panics
///
/// Panics if `read_spv` fails to read the SPIR-V data, which can happen if the
/// data is not valid SPIR-V or not correctly aligned/padded for `u32` reading.
pub fn load_shader_module(device: &Device, spirv_bytes: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
    // ash::util::read_spv expects a Read + Seek trait object, and handles converting bytes to Vec<u32>.
    // It will panic if the slice length is not a multiple of 4.
    // Ensure your SPIR-V byte data is correctly padded if necessary, though include_bytes! should be fine.
    let mut cursor = Cursor::new(spirv_bytes);
    let code_u32 = read_spv(&mut cursor)
        .expect("Failed to read SPIR-V shader from bytes. Ensure data is valid and correctly aligned/padded for u32 words.");

    let create_info = vk::ShaderModuleCreateInfo::builder().code(&code_u32);

    unsafe { device.create_shader_module(&create_info, None) }
}

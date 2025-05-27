use serde::{Serialize, Deserialize};
use smithay::reexports::input::AccelProfile as LibinputAccelProfile;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AccelProfile {
    Adaptive,
    Flat,
    Custom, // If libinput supports custom curves via points
}

impl From<LibinputAccelProfile> for AccelProfile {
    fn from(profile: LibinputAccelProfile) -> Self {
        match profile {
            LibinputAccelProfile::Adaptive => AccelProfile::Adaptive,
            LibinputAccelProfile::Flat => AccelProfile::Flat,
            _ => AccelProfile::Custom, // Or handle other specific profiles if they exist
        }
    }
}

impl From<AccelProfile> for LibinputAccelProfile {
    fn from(profile: AccelProfile) -> Self {
        match profile {
            AccelProfile::Adaptive => LibinputAccelProfile::Adaptive,
            AccelProfile::Flat => LibinputAccelProfile::Flat,
            AccelProfile::Custom => LibinputAccelProfile::Flat, // Fallback, libinput might not map 'Custom' directly
        }
    }
}

// Wrapper for device identification if needed, e.g., by sysname or name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PointerDeviceIdentifier {
    pub name: String, // e.g., "Logitech G Pro X Superlight"
    pub syspath: String, // e.g., "/sys/devices/pci0000:00/0000:00:01.2/.../input/inputXX"
}

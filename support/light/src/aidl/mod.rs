pub mod ILights;
pub mod HwLight;
pub mod HwLightState;
pub mod FlashMode;
pub mod BrightnessMode;
pub mod LightType;

pub(crate) mod mangled {
    pub use super::BrightnessMode::BrightnessMode as _7_android_8_hardware_5_light_14_BrightnessMode;
    pub use super::FlashMode::FlashMode as _7_android_8_hardware_5_light_9_FlashMode;
    pub use super::HwLight::HwLight as _7_android_8_hardware_5_light_7_HwLight;
    pub use super::HwLightState::HwLightState as _7_android_8_hardware_5_light_12_HwLightState;
    pub use super::LightType::LightType as _7_android_8_hardware_5_light_9_LightType;
}

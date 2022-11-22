use log::{error, info};

pub mod hidl;
pub mod aidl;

// Abstraction around HIDL / AIDL implementations.

#[derive(Debug)]
pub enum LightType {
    Backlight,
    Buttons,
    Keyboard,
}

// HIDL constants.
const FLASH_NONE: i32 = 0;
const BRIGHTNESS_USER: i32 = 0;
const TYPE_BACKLIGHT: i32 = 0;
const TYPE_KEYBOARD: i32 = 1;
const TYPE_BUTTONS: i32 = 2;

use aidl::LightType::LightType as AidlLightType;

impl LightType {
    fn as_hidl(&self) -> i32 {
        match self {
            Self::Backlight => TYPE_BACKLIGHT,
            Self::Buttons => TYPE_BUTTONS,
            Self::Keyboard => TYPE_KEYBOARD,
        }
    }

    fn as_aidl(&self) -> AidlLightType {
        match self {
            Self::Backlight => AidlLightType::BACKLIGHT,
            Self::Buttons => AidlLightType::BUTTONS,
            Self::Keyboard => AidlLightType::KEYBOARD,
        }
    }
}

pub trait LightInterface {
    fn is_alive(&self) -> bool;
    fn set_light_color(&self, light: LightType, color: u32) -> Result<i32, ()>;
    // brightness value: 0-100
    fn set_backlight_brightness(&self, value: f32) -> Result<i32, ()> {
        // Set the backlight for main screen.
        let brightness = ((value as f32 * 255.0) / (100.0)).round() as u32;
        let color = 0xff00_0000 + (brightness << 16) + (brightness << 8) + brightness;

        if self.is_alive() {
            self.set_light_color(LightType::Backlight, color)
        } else {
            Err(())
        }
    }
}

// HIDL implementation of the generic light interface.
struct HidlImpl {
    service: crate::hidl::ILight,
}

impl HidlImpl {
    fn get_service() -> Option<Self> {
        crate::hidl::ILight::get_service("default").map(|service| Self { service })
    }
}

impl LightInterface for HidlImpl {
    fn is_alive(&self) -> bool {
        self.service.is_alive()
    }

    fn set_light_color(&self, light: LightType, color: u32) -> Result<i32, ()> {

        let light_state = crate::hidl::LightState {
            color,
            flash_mode: FLASH_NONE,
            flash_on_ms: 0,
            flash_off_ms: 0,
            brightness_mode: BRIGHTNESS_USER,
        };

        self.service.set_light(light.as_hidl(), &light_state).map_err(|_| ())
    }
}

// AIDL implementation of the generic light interface.
struct AidlImpl {
    service: binder::Strong<dyn aidl::ILights::ILights>,
    lights: Vec<aidl::HwLight::HwLight>,
}

impl AidlImpl {
    fn get_service() -> Option<Self> {
        const SERVICE_NAME: &str = "android.hardware.light.ILights/default";
        if let Ok(true) = binder::is_declared(SERVICE_NAME) {
            match binder::wait_for_interface::<dyn aidl::ILights::ILights>(SERVICE_NAME) {
                Ok(service) => {
                    let lights = service.getLights().unwrap_or_default();
                    Some(Self { service, lights })
                }
                Err(err) => {
                    error!("Failure in wait_for_interface::<ILights>: {}", err);
                    None
                }
            }
        } else {
            error!("No AIDL service declared for: {}", SERVICE_NAME);
            None
        }
    }
}

impl LightInterface for AidlImpl {
    fn is_alive(&self) -> bool {
        use binder::binder_impl::IBinderInternal;

        self.service.as_binder().is_binder_alive()
    }

    fn set_light_color(&self, light: LightType, color: u32) -> Result<i32, ()> {

        let light_state = crate::aidl::HwLightState::HwLightState {
            color: color as _,
            flashMode: crate::aidl::FlashMode::FlashMode::NONE,
            flashOnMs: 0,
            flashOffMs: 0,
            brightnessMode: crate::aidl::BrightnessMode::BrightnessMode::USER,
        };

        // Find the light id for this type.
        let aidl_type = light.as_aidl();
        let mut id = -1;
        for light in &self.lights {
            if light.r#type == aidl_type {
                id = light.id;
                break;
            }
        }

        self.service.setLightState(id, &light_state)
            .map(|_| 0).map_err(|_| ())
    }
}

// Native implementation for backlight brightness
extern "C" {
    fn nativeGetDisplayBrightnessSupport() -> bool;
    fn nativeSetDisplayBrightness(
        sdr_brightness: f32,
        sdr_brightness_nits: f32,
        display_brightness: f32,
        display_brightness_nits: f32) -> bool;

}

pub struct LightService {
    backend: Box<dyn LightInterface>
}

impl LightService {
    // Tries HIDL, AIDL and fallback on native implementation.
    pub fn get_service() -> Option<Self> {
        if let Some(hidl) = HidlImpl::get_service() {
            info!("Using the HIDL backend for the LightService");
            Some(Self { backend: Box::new(hidl) })
        } else if let Some(aidl) = AidlImpl::get_service() {
            info!("Using the HIDL backend for the LightService");
            Some(Self { backend: Box::new(aidl) })
        } else {
            error!("No backend available for the LightService");
            None
        }
    }
}

impl LightInterface for LightService {
    fn is_alive(&self) -> bool {
        self.backend.is_alive()
    }

    fn set_light_color(&self, light: LightType, color: u32) -> Result<i32, ()> {
        self.backend.set_light_color(light, color)
    }
}

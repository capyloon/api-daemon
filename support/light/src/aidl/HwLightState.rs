#![forbid(unsafe_code)]
// #![rustfmt::skip]
#[derive(Debug)]
pub struct r#HwLightState {
  pub r#color: i32,
  pub r#flashMode: crate::aidl::FlashMode::mangled::_7_android_8_hardware_5_light_9_FlashMode,
  pub r#flashOnMs: i32,
  pub r#flashOffMs: i32,
  pub r#brightnessMode: crate::aidl::BrightnessMode::mangled::_7_android_8_hardware_5_light_14_BrightnessMode,
}
impl Default for r#HwLightState {
  fn default() -> Self {
    Self {
      r#color: 0,
      r#flashMode: Default::default(),
      r#flashOnMs: 0,
      r#flashOffMs: 0,
      r#brightnessMode: Default::default(),
    }
  }
}
impl binder::Parcelable for r#HwLightState {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#color)?;
      subparcel.write(&self.r#flashMode)?;
      subparcel.write(&self.r#flashOnMs)?;
      subparcel.write(&self.r#flashOffMs)?;
      subparcel.write(&self.r#brightnessMode)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#color = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#flashMode = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#flashOnMs = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#flashOffMs = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#brightnessMode = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#HwLightState);
binder::impl_deserialize_for_parcelable!(r#HwLightState);
impl binder::binder_impl::ParcelableMetadata for r#HwLightState {
  fn get_descriptor() -> &'static str { "android.hardware.light.HwLightState" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#HwLightState as _7_android_8_hardware_5_light_12_HwLightState;
}

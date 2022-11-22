#![forbid(unsafe_code)]
// #![rustfmt::skip]
#[derive(Debug)]
pub struct r#HwLight {
  pub r#id: i32,
  pub r#ordinal: i32,
  pub r#type: crate::aidl::LightType::mangled::_7_android_8_hardware_5_light_9_LightType,
}
impl Default for r#HwLight {
  fn default() -> Self {
    Self {
      r#id: 0,
      r#ordinal: 0,
      r#type: Default::default(),
    }
  }
}
impl binder::Parcelable for r#HwLight {
  fn write_to_parcel(&self, parcel: &mut binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_write(|subparcel| {
      subparcel.write(&self.r#id)?;
      subparcel.write(&self.r#ordinal)?;
      subparcel.write(&self.r#type)?;
      Ok(())
    })
  }
  fn read_from_parcel(&mut self, parcel: &binder::binder_impl::BorrowedParcel) -> std::result::Result<(), binder::StatusCode> {
    parcel.sized_read(|subparcel| {
      if subparcel.has_more_data() {
        self.r#id = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#ordinal = subparcel.read()?;
      }
      if subparcel.has_more_data() {
        self.r#type = subparcel.read()?;
      }
      Ok(())
    })
  }
}
binder::impl_serialize_for_parcelable!(r#HwLight);
binder::impl_deserialize_for_parcelable!(r#HwLight);
impl binder::binder_impl::ParcelableMetadata for r#HwLight {
  fn get_descriptor() -> &'static str { "android.hardware.light.HwLight" }
  fn get_stability(&self) -> binder::binder_impl::Stability { binder::binder_impl::Stability::Vintf }
}
pub(crate) mod mangled {
 pub use super::r#HwLight as _7_android_8_hardware_5_light_7_HwLight;
}

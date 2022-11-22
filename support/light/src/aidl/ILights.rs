#![forbid(unsafe_code)]
// #![rustfmt::skip]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#[allow(unused_imports)] use binder::binder_impl::IBinderInternal;
use binder::declare_binder_interface;
declare_binder_interface! {
  ILights["android.hardware.light.ILights"] {
    native: BnLights(on_transact),
    proxy: BpLights {
      cached_version: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1),
      cached_hash: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None)
    },
    async: ILightsAsync,
    stability: binder::binder_impl::Stability::Vintf,
  }
}
pub trait ILights: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.light.ILights" }
  fn r#setLightState(&self, _arg_id: i32, _arg_state: &crate::aidl::mangled::_7_android_8_hardware_5_light_12_HwLightState) -> binder::Result<()>;
  fn r#getLights(&self) -> binder::Result<Vec<crate::aidl::mangled::_7_android_8_hardware_5_light_7_HwLight>>;
  fn r#getInterfaceVersion(&self) -> binder::Result<i32> {
    Ok(VERSION)
  }
  fn r#getInterfaceHash(&self) -> binder::Result<String> {
    Ok(HASH.into())
  }
  fn getDefaultImpl() -> ILightsDefaultRef where Self: Sized {
    DEFAULT_IMPL.lock().unwrap().clone()
  }
  fn setDefaultImpl(d: ILightsDefaultRef) -> ILightsDefaultRef where Self: Sized {
    std::mem::replace(&mut *DEFAULT_IMPL.lock().unwrap(), d)
  }
}
pub trait ILightsAsync<P>: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.light.ILights" }
  fn r#setLightState<'a>(&'a self, _arg_id: i32, _arg_state: &'a crate::aidl::mangled::_7_android_8_hardware_5_light_12_HwLightState) -> binder::BoxFuture<'a, binder::Result<()>>;
  fn r#getLights<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::aidl::mangled::_7_android_8_hardware_5_light_7_HwLight>>>;
  fn r#getInterfaceVersion<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<i32>> {
    Box::pin(async move { Ok(VERSION) })
  }
  fn r#getInterfaceHash<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>> {
    Box::pin(async move { Ok(HASH.into()) })
  }
}
#[::async_trait::async_trait]
pub trait ILightsAsyncServer: binder::Interface + Send {
  fn get_descriptor() -> &'static str where Self: Sized { "android.hardware.light.ILights" }
  async fn r#setLightState(&self, _arg_id: i32, _arg_state: &crate::aidl::mangled::_7_android_8_hardware_5_light_12_HwLightState) -> binder::Result<()>;
  async fn r#getLights(&self) -> binder::Result<Vec<crate::aidl::mangled::_7_android_8_hardware_5_light_7_HwLight>>;
}
impl BnLights {
  /// Create a new async binder service.
  pub fn new_async_binder<T, R>(inner: T, rt: R, features: binder::BinderFeatures) -> binder::Strong<dyn ILights>
  where
    T: ILightsAsyncServer + binder::Interface + Send + Sync + 'static,
    R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
  {
    struct Wrapper<T, R> {
      _inner: T,
      _rt: R,
    }
    impl<T, R> binder::Interface for Wrapper<T, R> where T: binder::Interface, R: Send + Sync {
      fn as_binder(&self) -> binder::SpIBinder { self._inner.as_binder() }
      fn dump(&self, _file: &std::fs::File, _args: &[&std::ffi::CStr]) -> std::result::Result<(), binder::StatusCode> { self._inner.dump(_file, _args) }
    }
    impl<T, R> ILights for Wrapper<T, R>
    where
      T: ILightsAsyncServer + Send + Sync + 'static,
      R: binder::binder_impl::BinderAsyncRuntime + Send + Sync + 'static,
    {
      fn r#setLightState(&self, _arg_id: i32, _arg_state: &crate::aidl::mangled::_7_android_8_hardware_5_light_12_HwLightState) -> binder::Result<()> {
        self._rt.block_on(self._inner.r#setLightState(_arg_id, _arg_state))
      }
      fn r#getLights(&self) -> binder::Result<Vec<crate::aidl::mangled::_7_android_8_hardware_5_light_7_HwLight>> {
        self._rt.block_on(self._inner.r#getLights())
      }
    }
    let wrapped = Wrapper { _inner: inner, _rt: rt };
    Self::new_binder(wrapped, features)
  }
}
pub trait ILightsDefault: Send + Sync {
  fn r#setLightState(&self, _arg_id: i32, _arg_state: &crate::aidl::mangled::_7_android_8_hardware_5_light_12_HwLightState) -> binder::Result<()> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
  fn r#getLights(&self) -> binder::Result<Vec<crate::aidl::mangled::_7_android_8_hardware_5_light_7_HwLight>> {
    Err(binder::StatusCode::UNKNOWN_TRANSACTION.into())
  }
}
pub mod transactions {
  pub const r#setLightState: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 0;
  pub const r#getLights: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 1;
  pub const r#getInterfaceVersion: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 16777214;
  pub const r#getInterfaceHash: binder::binder_impl::TransactionCode = binder::binder_impl::FIRST_CALL_TRANSACTION + 16777213;
}
pub type ILightsDefaultRef = Option<std::sync::Arc<dyn ILightsDefault>>;
use lazy_static::lazy_static;
lazy_static! {
  static ref DEFAULT_IMPL: std::sync::Mutex<ILightsDefaultRef> = std::sync::Mutex::new(None);
}
pub const VERSION: i32 = 2;
pub const HASH: &str = "c8b1e8ebb88c57dcb2c350a8d9b722e77dd864c8";
impl BpLights {
  fn build_parcel_setLightState(&self, _arg_id: i32, _arg_state: &crate::aidl::mangled::_7_android_8_hardware_5_light_12_HwLightState) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    aidl_data.write(&_arg_id)?;
    aidl_data.write(_arg_state)?;
    Ok(aidl_data)
  }
  fn read_response_setLightState(&self, _arg_id: i32, _arg_state: &crate::aidl::mangled::_7_android_8_hardware_5_light_12_HwLightState, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<()> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as ILights>::getDefaultImpl() {
        return _aidl_default_impl.r#setLightState(_arg_id, _arg_state);
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    Ok(())
  }
  fn build_parcel_getLights(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getLights(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<Vec<crate::aidl::mangled::_7_android_8_hardware_5_light_7_HwLight>> {
    if let Err(binder::StatusCode::UNKNOWN_TRANSACTION) = _aidl_reply {
      if let Some(_aidl_default_impl) = <Self as ILights>::getDefaultImpl() {
        return _aidl_default_impl.r#getLights();
      }
    }
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: Vec<crate::aidl::mangled::_7_android_8_hardware_5_light_7_HwLight> = _aidl_reply.read()?;
    Ok(_aidl_return)
  }
  fn build_parcel_getInterfaceVersion(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getInterfaceVersion(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<i32> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: i32 = _aidl_reply.read()?;
    self.cached_version.store(_aidl_return, std::sync::atomic::Ordering::Relaxed);
    Ok(_aidl_return)
  }
  fn build_parcel_getInterfaceHash(&self) -> binder::Result<binder::binder_impl::Parcel> {
    let mut aidl_data = self.binder.prepare_transact()?;
    Ok(aidl_data)
  }
  fn read_response_getInterfaceHash(&self, _aidl_reply: std::result::Result<binder::binder_impl::Parcel, binder::StatusCode>) -> binder::Result<String> {
    let _aidl_reply = _aidl_reply?;
    let _aidl_status: binder::Status = _aidl_reply.read()?;
    if !_aidl_status.is_ok() { return Err(_aidl_status); }
    let _aidl_return: String = _aidl_reply.read()?;
    *self.cached_hash.lock().unwrap() = Some(_aidl_return.clone());
    Ok(_aidl_return)
  }
}
impl ILights for BpLights {
  fn r#setLightState(&self, _arg_id: i32, _arg_state: &crate::aidl::mangled::_7_android_8_hardware_5_light_12_HwLightState) -> binder::Result<()> {
    let _aidl_data = self.build_parcel_setLightState(_arg_id, _arg_state)?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#setLightState, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_setLightState(_arg_id, _arg_state, _aidl_reply)
  }
  fn r#getLights(&self) -> binder::Result<Vec<crate::aidl::mangled::_7_android_8_hardware_5_light_7_HwLight>> {
    let _aidl_data = self.build_parcel_getLights()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getLights, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getLights(_aidl_reply)
  }
  fn r#getInterfaceVersion(&self) -> binder::Result<i32> {
    let _aidl_version = self.cached_version.load(std::sync::atomic::Ordering::Relaxed);
    if _aidl_version != -1 { return Ok(_aidl_version); }
    let _aidl_data = self.build_parcel_getInterfaceVersion()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getInterfaceVersion, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getInterfaceVersion(_aidl_reply)
  }
  fn r#getInterfaceHash(&self) -> binder::Result<String> {
    {
      let _aidl_hash_lock = self.cached_hash.lock().unwrap();
      if let Some(ref _aidl_hash) = *_aidl_hash_lock {
        return Ok(_aidl_hash.clone());
      }
    }
    let _aidl_data = self.build_parcel_getInterfaceHash()?;
    let _aidl_reply = self.binder.submit_transact(transactions::r#getInterfaceHash, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL);
    self.read_response_getInterfaceHash(_aidl_reply)
  }
}
impl<P: binder::BinderAsyncPool> ILightsAsync<P> for BpLights {
  fn r#setLightState<'a>(&'a self, _arg_id: i32, _arg_state: &'a crate::aidl::mangled::_7_android_8_hardware_5_light_12_HwLightState) -> binder::BoxFuture<'a, binder::Result<()>> {
    let _aidl_data = match self.build_parcel_setLightState(_arg_id, _arg_state) {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#setLightState, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_setLightState(_arg_id, _arg_state, _aidl_reply)
      }
    )
  }
  fn r#getLights<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<Vec<crate::aidl::mangled::_7_android_8_hardware_5_light_7_HwLight>>> {
    let _aidl_data = match self.build_parcel_getLights() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getLights, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getLights(_aidl_reply)
      }
    )
  }
  fn r#getInterfaceVersion<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<i32>> {
    let _aidl_version = self.cached_version.load(std::sync::atomic::Ordering::Relaxed);
    if _aidl_version != -1 { return Box::pin(std::future::ready(Ok(_aidl_version))); }
    let _aidl_data = match self.build_parcel_getInterfaceVersion() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getInterfaceVersion, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getInterfaceVersion(_aidl_reply)
      }
    )
  }
  fn r#getInterfaceHash<'a>(&'a self) -> binder::BoxFuture<'a, binder::Result<String>> {
    {
      let _aidl_hash_lock = self.cached_hash.lock().unwrap();
      if let Some(ref _aidl_hash) = *_aidl_hash_lock {
        return Box::pin(std::future::ready(Ok(_aidl_hash.clone())));
      }
    }
    let _aidl_data = match self.build_parcel_getInterfaceHash() {
      Ok(_aidl_data) => _aidl_data,
      Err(err) => return Box::pin(std::future::ready(Err(err))),
    };
    let binder = self.binder.clone();
    P::spawn(
      move || binder.submit_transact(transactions::r#getInterfaceHash, _aidl_data, binder::binder_impl::FLAG_PRIVATE_LOCAL),
      move |_aidl_reply| async move {
        self.read_response_getInterfaceHash(_aidl_reply)
      }
    )
  }
}
impl ILights for binder::binder_impl::Binder<BnLights> {
  fn r#setLightState(&self, _arg_id: i32, _arg_state: &crate::aidl::mangled::_7_android_8_hardware_5_light_12_HwLightState) -> binder::Result<()> { self.0.r#setLightState(_arg_id, _arg_state) }
  fn r#getLights(&self) -> binder::Result<Vec<crate::aidl::mangled::_7_android_8_hardware_5_light_7_HwLight>> { self.0.r#getLights() }
  fn r#getInterfaceVersion(&self) -> binder::Result<i32> { self.0.r#getInterfaceVersion() }
  fn r#getInterfaceHash(&self) -> binder::Result<String> { self.0.r#getInterfaceHash() }
}
fn on_transact(_aidl_service: &dyn ILights, _aidl_code: binder::binder_impl::TransactionCode, _aidl_data: &binder::binder_impl::BorrowedParcel<'_>, _aidl_reply: &mut binder::binder_impl::BorrowedParcel<'_>) -> std::result::Result<(), binder::StatusCode> {
  match _aidl_code {
    transactions::r#setLightState => {
      let _arg_id: i32 = _aidl_data.read()?;
      let _arg_state: crate::aidl::HwLightState::mangled::_7_android_8_hardware_5_light_12_HwLightState = _aidl_data.read()?;
      let _aidl_return = _aidl_service.r#setLightState(_arg_id, &_arg_state);
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getLights => {
      let _aidl_return = _aidl_service.r#getLights();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getInterfaceVersion => {
      let _aidl_return = _aidl_service.r#getInterfaceVersion();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    transactions::r#getInterfaceHash => {
      let _aidl_return = _aidl_service.r#getInterfaceHash();
      match &_aidl_return {
        Ok(_aidl_return) => {
          _aidl_reply.write(&binder::Status::from(binder::StatusCode::OK))?;
          _aidl_reply.write(_aidl_return)?;
        }
        Err(_aidl_status) => _aidl_reply.write(_aidl_status)?
      }
      Ok(())
    }
    _ => Err(binder::StatusCode::UNKNOWN_TRANSACTION)
  }
}
pub(crate) mod mangled {
 pub use super::r#ILights as _7_android_8_hardware_5_light_7_ILights;
}

#[cfg(all(feature = "UI_Composition", feature = "Win32_Foundation"))]
pub trait ICompositionCapabilitiesInteropFactory_Impl: Sized {
    fn GetForWindow(&mut self, hwnd: super::super::super::Foundation::HWND) -> ::windows::core::Result<super::super::super::super::UI::Composition::CompositionCapabilities>;
}
#[cfg(all(feature = "UI_Composition", feature = "Win32_Foundation"))]
impl ::windows::core::RuntimeName for ICompositionCapabilitiesInteropFactory {
    const NAME: &'static str = "";
}
#[cfg(all(feature = "UI_Composition", feature = "Win32_Foundation"))]
impl ICompositionCapabilitiesInteropFactory_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionCapabilitiesInteropFactory_Impl, const OFFSET: isize>() -> ICompositionCapabilitiesInteropFactory_Vtbl {
        unsafe extern "system" fn GetForWindow<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionCapabilitiesInteropFactory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, hwnd: super::super::super::Foundation::HWND, result: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetForWindow(::core::mem::transmute_copy(&hwnd)) {
                ::core::result::Result::Ok(ok__) => {
                    *result = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ::windows::core::IInspectableVtbl::new::<Identity, ICompositionCapabilitiesInteropFactory, OFFSET>(),
            GetForWindow: GetForWindow::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ICompositionCapabilitiesInteropFactory as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ICompositionDrawingSurfaceInterop_Impl: Sized {
    fn BeginDraw(&mut self, updaterect: *const super::super::super::Foundation::RECT, iid: *const ::windows::core::GUID, updateobject: *mut *mut ::core::ffi::c_void, updateoffset: *mut super::super::super::Foundation::POINT) -> ::windows::core::Result<()>;
    fn EndDraw(&mut self) -> ::windows::core::Result<()>;
    fn Resize(&mut self, sizepixels: &super::super::super::Foundation::SIZE) -> ::windows::core::Result<()>;
    fn Scroll(&mut self, scrollrect: *const super::super::super::Foundation::RECT, cliprect: *const super::super::super::Foundation::RECT, offsetx: i32, offsety: i32) -> ::windows::core::Result<()>;
    fn ResumeDraw(&mut self) -> ::windows::core::Result<()>;
    fn SuspendDraw(&mut self) -> ::windows::core::Result<()>;
}
#[cfg(feature = "Win32_Foundation")]
impl ICompositionDrawingSurfaceInterop_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionDrawingSurfaceInterop_Impl, const OFFSET: isize>() -> ICompositionDrawingSurfaceInterop_Vtbl {
        unsafe extern "system" fn BeginDraw<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionDrawingSurfaceInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, updaterect: *const super::super::super::Foundation::RECT, iid: *const ::windows::core::GUID, updateobject: *mut *mut ::core::ffi::c_void, updateoffset: *mut super::super::super::Foundation::POINT) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).BeginDraw(::core::mem::transmute_copy(&updaterect), ::core::mem::transmute_copy(&iid), ::core::mem::transmute_copy(&updateobject), ::core::mem::transmute_copy(&updateoffset)).into()
        }
        unsafe extern "system" fn EndDraw<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionDrawingSurfaceInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).EndDraw().into()
        }
        unsafe extern "system" fn Resize<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionDrawingSurfaceInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, sizepixels: super::super::super::Foundation::SIZE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Resize(::core::mem::transmute_copy(&sizepixels)).into()
        }
        unsafe extern "system" fn Scroll<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionDrawingSurfaceInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, scrollrect: *const super::super::super::Foundation::RECT, cliprect: *const super::super::super::Foundation::RECT, offsetx: i32, offsety: i32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Scroll(::core::mem::transmute_copy(&scrollrect), ::core::mem::transmute_copy(&cliprect), ::core::mem::transmute_copy(&offsetx), ::core::mem::transmute_copy(&offsety)).into()
        }
        unsafe extern "system" fn ResumeDraw<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionDrawingSurfaceInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ResumeDraw().into()
        }
        unsafe extern "system" fn SuspendDraw<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionDrawingSurfaceInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SuspendDraw().into()
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            BeginDraw: BeginDraw::<Identity, Impl, OFFSET>,
            EndDraw: EndDraw::<Identity, Impl, OFFSET>,
            Resize: Resize::<Identity, Impl, OFFSET>,
            Scroll: Scroll::<Identity, Impl, OFFSET>,
            ResumeDraw: ResumeDraw::<Identity, Impl, OFFSET>,
            SuspendDraw: SuspendDraw::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ICompositionDrawingSurfaceInterop as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ICompositionDrawingSurfaceInterop2_Impl: Sized + ICompositionDrawingSurfaceInterop_Impl {
    fn CopySurface(&mut self, destinationresource: &::core::option::Option<::windows::core::IUnknown>, destinationoffsetx: i32, destinationoffsety: i32, sourcerectangle: *const super::super::super::Foundation::RECT) -> ::windows::core::Result<()>;
}
#[cfg(feature = "Win32_Foundation")]
impl ICompositionDrawingSurfaceInterop2_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionDrawingSurfaceInterop2_Impl, const OFFSET: isize>() -> ICompositionDrawingSurfaceInterop2_Vtbl {
        unsafe extern "system" fn CopySurface<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionDrawingSurfaceInterop2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, destinationresource: *mut ::core::ffi::c_void, destinationoffsetx: i32, destinationoffsety: i32, sourcerectangle: *const super::super::super::Foundation::RECT) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).CopySurface(::core::mem::transmute(&destinationresource), ::core::mem::transmute_copy(&destinationoffsetx), ::core::mem::transmute_copy(&destinationoffsety), ::core::mem::transmute_copy(&sourcerectangle)).into()
        }
        Self { base: ICompositionDrawingSurfaceInterop_Vtbl::new::<Identity, Impl, OFFSET>(), CopySurface: CopySurface::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ICompositionDrawingSurfaceInterop2 as ::windows::core::Interface>::IID || iid == &<ICompositionDrawingSurfaceInterop as ::windows::core::Interface>::IID
    }
}
pub trait ICompositionGraphicsDeviceInterop_Impl: Sized {
    fn GetRenderingDevice(&mut self) -> ::windows::core::Result<::windows::core::IUnknown>;
    fn SetRenderingDevice(&mut self, value: &::core::option::Option<::windows::core::IUnknown>) -> ::windows::core::Result<()>;
}
impl ICompositionGraphicsDeviceInterop_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionGraphicsDeviceInterop_Impl, const OFFSET: isize>() -> ICompositionGraphicsDeviceInterop_Vtbl {
        unsafe extern "system" fn GetRenderingDevice<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionGraphicsDeviceInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, value: *mut *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetRenderingDevice() {
                ::core::result::Result::Ok(ok__) => {
                    *value = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn SetRenderingDevice<Identity: ::windows::core::IUnknownImpl, Impl: ICompositionGraphicsDeviceInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, value: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetRenderingDevice(::core::mem::transmute(&value)).into()
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            GetRenderingDevice: GetRenderingDevice::<Identity, Impl, OFFSET>,
            SetRenderingDevice: SetRenderingDevice::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ICompositionGraphicsDeviceInterop as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "UI_Composition_Desktop", feature = "Win32_Foundation"))]
pub trait ICompositorDesktopInterop_Impl: Sized {
    fn CreateDesktopWindowTarget(&mut self, hwndtarget: super::super::super::Foundation::HWND, istopmost: super::super::super::Foundation::BOOL) -> ::windows::core::Result<super::super::super::super::UI::Composition::Desktop::DesktopWindowTarget>;
    fn EnsureOnThread(&mut self, threadid: u32) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "UI_Composition_Desktop", feature = "Win32_Foundation"))]
impl ICompositorDesktopInterop_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ICompositorDesktopInterop_Impl, const OFFSET: isize>() -> ICompositorDesktopInterop_Vtbl {
        unsafe extern "system" fn CreateDesktopWindowTarget<Identity: ::windows::core::IUnknownImpl, Impl: ICompositorDesktopInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, hwndtarget: super::super::super::Foundation::HWND, istopmost: super::super::super::Foundation::BOOL, result: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDesktopWindowTarget(::core::mem::transmute_copy(&hwndtarget), ::core::mem::transmute_copy(&istopmost)) {
                ::core::result::Result::Ok(ok__) => {
                    *result = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn EnsureOnThread<Identity: ::windows::core::IUnknownImpl, Impl: ICompositorDesktopInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, threadid: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).EnsureOnThread(::core::mem::transmute_copy(&threadid)).into()
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            CreateDesktopWindowTarget: CreateDesktopWindowTarget::<Identity, Impl, OFFSET>,
            EnsureOnThread: EnsureOnThread::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ICompositorDesktopInterop as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "UI_Composition", feature = "Win32_Foundation"))]
pub trait ICompositorInterop_Impl: Sized {
    fn CreateCompositionSurfaceForHandle(&mut self, swapchain: super::super::super::Foundation::HANDLE) -> ::windows::core::Result<super::super::super::super::UI::Composition::ICompositionSurface>;
    fn CreateCompositionSurfaceForSwapChain(&mut self, swapchain: &::core::option::Option<::windows::core::IUnknown>) -> ::windows::core::Result<super::super::super::super::UI::Composition::ICompositionSurface>;
    fn CreateGraphicsDevice(&mut self, renderingdevice: &::core::option::Option<::windows::core::IUnknown>) -> ::windows::core::Result<super::super::super::super::UI::Composition::CompositionGraphicsDevice>;
}
#[cfg(all(feature = "UI_Composition", feature = "Win32_Foundation"))]
impl ICompositorInterop_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ICompositorInterop_Impl, const OFFSET: isize>() -> ICompositorInterop_Vtbl {
        unsafe extern "system" fn CreateCompositionSurfaceForHandle<Identity: ::windows::core::IUnknownImpl, Impl: ICompositorInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, swapchain: super::super::super::Foundation::HANDLE, result: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateCompositionSurfaceForHandle(::core::mem::transmute_copy(&swapchain)) {
                ::core::result::Result::Ok(ok__) => {
                    *result = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateCompositionSurfaceForSwapChain<Identity: ::windows::core::IUnknownImpl, Impl: ICompositorInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, swapchain: *mut ::core::ffi::c_void, result: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateCompositionSurfaceForSwapChain(::core::mem::transmute(&swapchain)) {
                ::core::result::Result::Ok(ok__) => {
                    *result = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateGraphicsDevice<Identity: ::windows::core::IUnknownImpl, Impl: ICompositorInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, renderingdevice: *mut ::core::ffi::c_void, result: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateGraphicsDevice(::core::mem::transmute(&renderingdevice)) {
                ::core::result::Result::Ok(ok__) => {
                    *result = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            CreateCompositionSurfaceForHandle: CreateCompositionSurfaceForHandle::<Identity, Impl, OFFSET>,
            CreateCompositionSurfaceForSwapChain: CreateCompositionSurfaceForSwapChain::<Identity, Impl, OFFSET>,
            CreateGraphicsDevice: CreateGraphicsDevice::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ICompositorInterop as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait IDesktopWindowTargetInterop_Impl: Sized {
    fn Hwnd(&mut self) -> ::windows::core::Result<super::super::super::Foundation::HWND>;
}
#[cfg(feature = "Win32_Foundation")]
impl IDesktopWindowTargetInterop_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: IDesktopWindowTargetInterop_Impl, const OFFSET: isize>() -> IDesktopWindowTargetInterop_Vtbl {
        unsafe extern "system" fn Hwnd<Identity: ::windows::core::IUnknownImpl, Impl: IDesktopWindowTargetInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, value: *mut super::super::super::Foundation::HWND) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).Hwnd() {
                ::core::result::Result::Ok(ok__) => {
                    *value = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(), Hwnd: Hwnd::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<IDesktopWindowTargetInterop as ::windows::core::Interface>::IID
    }
}
pub trait ISwapChainInterop_Impl: Sized {
    fn SetSwapChain(&mut self, swapchain: &::core::option::Option<::windows::core::IUnknown>) -> ::windows::core::Result<()>;
}
impl ISwapChainInterop_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ISwapChainInterop_Impl, const OFFSET: isize>() -> ISwapChainInterop_Vtbl {
        unsafe extern "system" fn SetSwapChain<Identity: ::windows::core::IUnknownImpl, Impl: ISwapChainInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, swapchain: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetSwapChain(::core::mem::transmute(&swapchain)).into()
        }
        Self { base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(), SetSwapChain: SetSwapChain::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ISwapChainInterop as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Input_Pointer", feature = "Win32_UI_WindowsAndMessaging"))]
pub trait IVisualInteractionSourceInterop_Impl: Sized {
    fn TryRedirectForManipulation(&mut self, pointerinfo: *const super::super::super::UI::Input::Pointer::POINTER_INFO) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Input_Pointer", feature = "Win32_UI_WindowsAndMessaging"))]
impl IVisualInteractionSourceInterop_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: IVisualInteractionSourceInterop_Impl, const OFFSET: isize>() -> IVisualInteractionSourceInterop_Vtbl {
        unsafe extern "system" fn TryRedirectForManipulation<Identity: ::windows::core::IUnknownImpl, Impl: IVisualInteractionSourceInterop_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, pointerinfo: *const super::super::super::UI::Input::Pointer::POINTER_INFO) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).TryRedirectForManipulation(::core::mem::transmute_copy(&pointerinfo)).into()
        }
        Self { base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(), TryRedirectForManipulation: TryRedirectForManipulation::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<IVisualInteractionSourceInterop as ::windows::core::Interface>::IID
    }
}

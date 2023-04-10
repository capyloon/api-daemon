pub trait ID2D1AnalysisTransform_Impl: Sized {
    fn ProcessAnalysisResults(&mut self, analysisdata: *const u8, analysisdatacount: u32) -> ::windows::core::Result<()>;
}
impl ID2D1AnalysisTransform_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1AnalysisTransform_Impl, const OFFSET: isize>() -> ID2D1AnalysisTransform_Vtbl {
        unsafe extern "system" fn ProcessAnalysisResults<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1AnalysisTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, analysisdata: *const u8, analysisdatacount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ProcessAnalysisResults(::core::mem::transmute_copy(&analysisdata), ::core::mem::transmute_copy(&analysisdatacount)).into()
        }
        Self { base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(), ProcessAnalysisResults: ProcessAnalysisResults::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1AnalysisTransform as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Dxgi_Common"))]
pub trait ID2D1Bitmap_Impl: Sized + ID2D1Resource_Impl + ID2D1Image_Impl {
    fn GetSize(&mut self) -> Common::D2D_SIZE_F;
    fn GetPixelSize(&mut self) -> Common::D2D_SIZE_U;
    fn GetPixelFormat(&mut self) -> Common::D2D1_PIXEL_FORMAT;
    fn GetDpi(&mut self, dpix: *mut f32, dpiy: *mut f32);
    fn CopyFromBitmap(&mut self, destpoint: *const Common::D2D_POINT_2U, bitmap: &::core::option::Option<ID2D1Bitmap>, srcrect: *const Common::D2D_RECT_U) -> ::windows::core::Result<()>;
    fn CopyFromRenderTarget(&mut self, destpoint: *const Common::D2D_POINT_2U, rendertarget: &::core::option::Option<ID2D1RenderTarget>, srcrect: *const Common::D2D_RECT_U) -> ::windows::core::Result<()>;
    fn CopyFromMemory(&mut self, dstrect: *const Common::D2D_RECT_U, srcdata: *const ::core::ffi::c_void, pitch: u32) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Dxgi_Common"))]
impl ID2D1Bitmap_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap_Impl, const OFFSET: isize>() -> ID2D1Bitmap_Vtbl {
        unsafe extern "system" fn GetSize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D_SIZE_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetSize()
        }
        unsafe extern "system" fn GetPixelSize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D_SIZE_U) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetPixelSize()
        }
        unsafe extern "system" fn GetPixelFormat<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D1_PIXEL_FORMAT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetPixelFormat()
        }
        unsafe extern "system" fn GetDpi<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dpix: *mut f32, dpiy: *mut f32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDpi(::core::mem::transmute_copy(&dpix), ::core::mem::transmute_copy(&dpiy))
        }
        unsafe extern "system" fn CopyFromBitmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, destpoint: *const Common::D2D_POINT_2U, bitmap: ::windows::core::RawPtr, srcrect: *const Common::D2D_RECT_U) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).CopyFromBitmap(::core::mem::transmute_copy(&destpoint), ::core::mem::transmute(&bitmap), ::core::mem::transmute_copy(&srcrect)).into()
        }
        unsafe extern "system" fn CopyFromRenderTarget<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, destpoint: *const Common::D2D_POINT_2U, rendertarget: ::windows::core::RawPtr, srcrect: *const Common::D2D_RECT_U) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).CopyFromRenderTarget(::core::mem::transmute_copy(&destpoint), ::core::mem::transmute(&rendertarget), ::core::mem::transmute_copy(&srcrect)).into()
        }
        unsafe extern "system" fn CopyFromMemory<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dstrect: *const Common::D2D_RECT_U, srcdata: *const ::core::ffi::c_void, pitch: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).CopyFromMemory(::core::mem::transmute_copy(&dstrect), ::core::mem::transmute_copy(&srcdata), ::core::mem::transmute_copy(&pitch)).into()
        }
        Self {
            base: ID2D1Image_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetSize: GetSize::<Identity, Impl, OFFSET>,
            GetPixelSize: GetPixelSize::<Identity, Impl, OFFSET>,
            GetPixelFormat: GetPixelFormat::<Identity, Impl, OFFSET>,
            GetDpi: GetDpi::<Identity, Impl, OFFSET>,
            CopyFromBitmap: CopyFromBitmap::<Identity, Impl, OFFSET>,
            CopyFromRenderTarget: CopyFromRenderTarget::<Identity, Impl, OFFSET>,
            CopyFromMemory: CopyFromMemory::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Bitmap as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Image as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common"))]
pub trait ID2D1Bitmap1_Impl: Sized + ID2D1Resource_Impl + ID2D1Image_Impl + ID2D1Bitmap_Impl {
    fn GetColorContext(&mut self, colorcontext: *mut ::core::option::Option<ID2D1ColorContext>);
    fn GetOptions(&mut self) -> D2D1_BITMAP_OPTIONS;
    fn GetSurface(&mut self) -> ::windows::core::Result<super::Dxgi::IDXGISurface>;
    fn Map(&mut self, options: D2D1_MAP_OPTIONS) -> ::windows::core::Result<D2D1_MAPPED_RECT>;
    fn Unmap(&mut self) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common"))]
impl ID2D1Bitmap1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap1_Impl, const OFFSET: isize>() -> ID2D1Bitmap1_Vtbl {
        unsafe extern "system" fn GetColorContext<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, colorcontext: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetColorContext(::core::mem::transmute_copy(&colorcontext))
        }
        unsafe extern "system" fn GetOptions<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_BITMAP_OPTIONS {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetOptions()
        }
        unsafe extern "system" fn GetSurface<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dxgisurface: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetSurface() {
                ::core::result::Result::Ok(ok__) => {
                    *dxgisurface = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn Map<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, options: D2D1_MAP_OPTIONS, mappedrect: *mut D2D1_MAPPED_RECT) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).Map(::core::mem::transmute_copy(&options)) {
                ::core::result::Result::Ok(ok__) => {
                    *mappedrect = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn Unmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Bitmap1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Unmap().into()
        }
        Self {
            base: ID2D1Bitmap_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetColorContext: GetColorContext::<Identity, Impl, OFFSET>,
            GetOptions: GetOptions::<Identity, Impl, OFFSET>,
            GetSurface: GetSurface::<Identity, Impl, OFFSET>,
            Map: Map::<Identity, Impl, OFFSET>,
            Unmap: Unmap::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Bitmap1 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Image as ::windows::core::Interface>::IID || iid == &<ID2D1Bitmap as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Foundation_Numerics")]
pub trait ID2D1BitmapBrush_Impl: Sized + ID2D1Resource_Impl + ID2D1Brush_Impl {
    fn SetExtendModeX(&mut self, extendmodex: D2D1_EXTEND_MODE);
    fn SetExtendModeY(&mut self, extendmodey: D2D1_EXTEND_MODE);
    fn SetInterpolationMode(&mut self, interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE);
    fn SetBitmap(&mut self, bitmap: &::core::option::Option<ID2D1Bitmap>);
    fn GetExtendModeX(&mut self) -> D2D1_EXTEND_MODE;
    fn GetExtendModeY(&mut self) -> D2D1_EXTEND_MODE;
    fn GetInterpolationMode(&mut self) -> D2D1_BITMAP_INTERPOLATION_MODE;
    fn GetBitmap(&mut self, bitmap: *mut ::core::option::Option<ID2D1Bitmap>);
}
#[cfg(feature = "Foundation_Numerics")]
impl ID2D1BitmapBrush_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush_Impl, const OFFSET: isize>() -> ID2D1BitmapBrush_Vtbl {
        unsafe extern "system" fn SetExtendModeX<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, extendmodex: D2D1_EXTEND_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetExtendModeX(::core::mem::transmute_copy(&extendmodex))
        }
        unsafe extern "system" fn SetExtendModeY<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, extendmodey: D2D1_EXTEND_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetExtendModeY(::core::mem::transmute_copy(&extendmodey))
        }
        unsafe extern "system" fn SetInterpolationMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetInterpolationMode(::core::mem::transmute_copy(&interpolationmode))
        }
        unsafe extern "system" fn SetBitmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bitmap: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetBitmap(::core::mem::transmute(&bitmap))
        }
        unsafe extern "system" fn GetExtendModeX<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_EXTEND_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetExtendModeX()
        }
        unsafe extern "system" fn GetExtendModeY<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_EXTEND_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetExtendModeY()
        }
        unsafe extern "system" fn GetInterpolationMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_BITMAP_INTERPOLATION_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetInterpolationMode()
        }
        unsafe extern "system" fn GetBitmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bitmap: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetBitmap(::core::mem::transmute_copy(&bitmap))
        }
        Self {
            base: ID2D1Brush_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetExtendModeX: SetExtendModeX::<Identity, Impl, OFFSET>,
            SetExtendModeY: SetExtendModeY::<Identity, Impl, OFFSET>,
            SetInterpolationMode: SetInterpolationMode::<Identity, Impl, OFFSET>,
            SetBitmap: SetBitmap::<Identity, Impl, OFFSET>,
            GetExtendModeX: GetExtendModeX::<Identity, Impl, OFFSET>,
            GetExtendModeY: GetExtendModeY::<Identity, Impl, OFFSET>,
            GetInterpolationMode: GetInterpolationMode::<Identity, Impl, OFFSET>,
            GetBitmap: GetBitmap::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1BitmapBrush as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Brush as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Foundation_Numerics")]
pub trait ID2D1BitmapBrush1_Impl: Sized + ID2D1Resource_Impl + ID2D1Brush_Impl + ID2D1BitmapBrush_Impl {
    fn SetInterpolationMode1(&mut self, interpolationmode: D2D1_INTERPOLATION_MODE);
    fn GetInterpolationMode1(&mut self) -> D2D1_INTERPOLATION_MODE;
}
#[cfg(feature = "Foundation_Numerics")]
impl ID2D1BitmapBrush1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush1_Impl, const OFFSET: isize>() -> ID2D1BitmapBrush1_Vtbl {
        unsafe extern "system" fn SetInterpolationMode1<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, interpolationmode: D2D1_INTERPOLATION_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetInterpolationMode1(::core::mem::transmute_copy(&interpolationmode))
        }
        unsafe extern "system" fn GetInterpolationMode1<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapBrush1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_INTERPOLATION_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetInterpolationMode1()
        }
        Self {
            base: ID2D1BitmapBrush_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetInterpolationMode1: SetInterpolationMode1::<Identity, Impl, OFFSET>,
            GetInterpolationMode1: GetInterpolationMode1::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1BitmapBrush1 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Brush as ::windows::core::Interface>::IID || iid == &<ID2D1BitmapBrush as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1BitmapRenderTarget_Impl: Sized + ID2D1Resource_Impl + ID2D1RenderTarget_Impl {
    fn GetBitmap(&mut self) -> ::windows::core::Result<ID2D1Bitmap>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1BitmapRenderTarget_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapRenderTarget_Impl, const OFFSET: isize>() -> ID2D1BitmapRenderTarget_Vtbl {
        unsafe extern "system" fn GetBitmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BitmapRenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bitmap: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetBitmap() {
                ::core::result::Result::Ok(ok__) => {
                    *bitmap = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1RenderTarget_Vtbl::new::<Identity, Impl, OFFSET>(), GetBitmap: GetBitmap::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1BitmapRenderTarget as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1BlendTransform_Impl: Sized + ID2D1TransformNode_Impl + ID2D1ConcreteTransform_Impl {
    fn SetDescription(&mut self, description: *const D2D1_BLEND_DESCRIPTION);
    fn GetDescription(&mut self, description: *mut D2D1_BLEND_DESCRIPTION);
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1BlendTransform_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BlendTransform_Impl, const OFFSET: isize>() -> ID2D1BlendTransform_Vtbl {
        unsafe extern "system" fn SetDescription<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BlendTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, description: *const D2D1_BLEND_DESCRIPTION) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetDescription(::core::mem::transmute_copy(&description))
        }
        unsafe extern "system" fn GetDescription<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BlendTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, description: *mut D2D1_BLEND_DESCRIPTION) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDescription(::core::mem::transmute_copy(&description))
        }
        Self {
            base: ID2D1ConcreteTransform_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetDescription: SetDescription::<Identity, Impl, OFFSET>,
            GetDescription: GetDescription::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1BlendTransform as ::windows::core::Interface>::IID || iid == &<ID2D1TransformNode as ::windows::core::Interface>::IID || iid == &<ID2D1ConcreteTransform as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1BorderTransform_Impl: Sized + ID2D1TransformNode_Impl + ID2D1ConcreteTransform_Impl {
    fn SetExtendModeX(&mut self, extendmode: D2D1_EXTEND_MODE);
    fn SetExtendModeY(&mut self, extendmode: D2D1_EXTEND_MODE);
    fn GetExtendModeX(&mut self) -> D2D1_EXTEND_MODE;
    fn GetExtendModeY(&mut self) -> D2D1_EXTEND_MODE;
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1BorderTransform_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BorderTransform_Impl, const OFFSET: isize>() -> ID2D1BorderTransform_Vtbl {
        unsafe extern "system" fn SetExtendModeX<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BorderTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, extendmode: D2D1_EXTEND_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetExtendModeX(::core::mem::transmute_copy(&extendmode))
        }
        unsafe extern "system" fn SetExtendModeY<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BorderTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, extendmode: D2D1_EXTEND_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetExtendModeY(::core::mem::transmute_copy(&extendmode))
        }
        unsafe extern "system" fn GetExtendModeX<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BorderTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_EXTEND_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetExtendModeX()
        }
        unsafe extern "system" fn GetExtendModeY<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BorderTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_EXTEND_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetExtendModeY()
        }
        Self {
            base: ID2D1ConcreteTransform_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetExtendModeX: SetExtendModeX::<Identity, Impl, OFFSET>,
            SetExtendModeY: SetExtendModeY::<Identity, Impl, OFFSET>,
            GetExtendModeX: GetExtendModeX::<Identity, Impl, OFFSET>,
            GetExtendModeY: GetExtendModeY::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1BorderTransform as ::windows::core::Interface>::IID || iid == &<ID2D1TransformNode as ::windows::core::Interface>::IID || iid == &<ID2D1ConcreteTransform as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1BoundsAdjustmentTransform_Impl: Sized + ID2D1TransformNode_Impl {
    fn SetOutputBounds(&mut self, outputbounds: *const super::super::Foundation::RECT);
    fn GetOutputBounds(&mut self, outputbounds: *mut super::super::Foundation::RECT);
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1BoundsAdjustmentTransform_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BoundsAdjustmentTransform_Impl, const OFFSET: isize>() -> ID2D1BoundsAdjustmentTransform_Vtbl {
        unsafe extern "system" fn SetOutputBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BoundsAdjustmentTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, outputbounds: *const super::super::Foundation::RECT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetOutputBounds(::core::mem::transmute_copy(&outputbounds))
        }
        unsafe extern "system" fn GetOutputBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1BoundsAdjustmentTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, outputbounds: *mut super::super::Foundation::RECT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetOutputBounds(::core::mem::transmute_copy(&outputbounds))
        }
        Self {
            base: ID2D1TransformNode_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetOutputBounds: SetOutputBounds::<Identity, Impl, OFFSET>,
            GetOutputBounds: GetOutputBounds::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1BoundsAdjustmentTransform as ::windows::core::Interface>::IID || iid == &<ID2D1TransformNode as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Foundation_Numerics")]
pub trait ID2D1Brush_Impl: Sized + ID2D1Resource_Impl {
    fn SetOpacity(&mut self, opacity: f32);
    fn SetTransform(&mut self, transform: *const super::super::super::Foundation::Numerics::Matrix3x2);
    fn GetOpacity(&mut self) -> f32;
    fn GetTransform(&mut self, transform: *mut super::super::super::Foundation::Numerics::Matrix3x2);
}
#[cfg(feature = "Foundation_Numerics")]
impl ID2D1Brush_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Brush_Impl, const OFFSET: isize>() -> ID2D1Brush_Vtbl {
        unsafe extern "system" fn SetOpacity<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Brush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, opacity: f32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetOpacity(::core::mem::transmute_copy(&opacity))
        }
        unsafe extern "system" fn SetTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Brush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, transform: *const super::super::super::Foundation::Numerics::Matrix3x2) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTransform(::core::mem::transmute_copy(&transform))
        }
        unsafe extern "system" fn GetOpacity<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Brush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> f32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetOpacity()
        }
        unsafe extern "system" fn GetTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Brush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, transform: *mut super::super::super::Foundation::Numerics::Matrix3x2) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTransform(::core::mem::transmute_copy(&transform))
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetOpacity: SetOpacity::<Identity, Impl, OFFSET>,
            SetTransform: SetTransform::<Identity, Impl, OFFSET>,
            GetOpacity: GetOpacity::<Identity, Impl, OFFSET>,
            GetTransform: GetTransform::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Brush as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1ColorContext_Impl: Sized + ID2D1Resource_Impl {
    fn GetColorSpace(&mut self) -> D2D1_COLOR_SPACE;
    fn GetProfileSize(&mut self) -> u32;
    fn GetProfile(&mut self, profile: *mut u8, profilesize: u32) -> ::windows::core::Result<()>;
}
impl ID2D1ColorContext_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ColorContext_Impl, const OFFSET: isize>() -> ID2D1ColorContext_Vtbl {
        unsafe extern "system" fn GetColorSpace<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ColorContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_COLOR_SPACE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetColorSpace()
        }
        unsafe extern "system" fn GetProfileSize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ColorContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetProfileSize()
        }
        unsafe extern "system" fn GetProfile<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ColorContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, profile: *mut u8, profilesize: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetProfile(::core::mem::transmute_copy(&profile), ::core::mem::transmute_copy(&profilesize)).into()
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetColorSpace: GetColorSpace::<Identity, Impl, OFFSET>,
            GetProfileSize: GetProfileSize::<Identity, Impl, OFFSET>,
            GetProfile: GetProfile::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1ColorContext as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Dxgi_Common"))]
pub trait ID2D1ColorContext1_Impl: Sized + ID2D1Resource_Impl + ID2D1ColorContext_Impl {
    fn GetColorContextType(&mut self) -> D2D1_COLOR_CONTEXT_TYPE;
    fn GetDXGIColorSpace(&mut self) -> super::Dxgi::Common::DXGI_COLOR_SPACE_TYPE;
    fn GetSimpleColorProfile(&mut self) -> ::windows::core::Result<D2D1_SIMPLE_COLOR_PROFILE>;
}
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Dxgi_Common"))]
impl ID2D1ColorContext1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ColorContext1_Impl, const OFFSET: isize>() -> ID2D1ColorContext1_Vtbl {
        unsafe extern "system" fn GetColorContextType<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ColorContext1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_COLOR_CONTEXT_TYPE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetColorContextType()
        }
        unsafe extern "system" fn GetDXGIColorSpace<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ColorContext1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> super::Dxgi::Common::DXGI_COLOR_SPACE_TYPE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDXGIColorSpace()
        }
        unsafe extern "system" fn GetSimpleColorProfile<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ColorContext1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, simpleprofile: *mut D2D1_SIMPLE_COLOR_PROFILE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetSimpleColorProfile() {
                ::core::result::Result::Ok(ok__) => {
                    *simpleprofile = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1ColorContext_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetColorContextType: GetColorContextType::<Identity, Impl, OFFSET>,
            GetDXGIColorSpace: GetDXGIColorSpace::<Identity, Impl, OFFSET>,
            GetSimpleColorProfile: GetSimpleColorProfile::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1ColorContext1 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1ColorContext as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1CommandList_Impl: Sized + ID2D1Resource_Impl + ID2D1Image_Impl {
    fn Stream(&mut self, sink: &::core::option::Option<ID2D1CommandSink>) -> ::windows::core::Result<()>;
    fn Close(&mut self) -> ::windows::core::Result<()>;
}
impl ID2D1CommandList_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandList_Impl, const OFFSET: isize>() -> ID2D1CommandList_Vtbl {
        unsafe extern "system" fn Stream<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandList_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, sink: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Stream(::core::mem::transmute(&sink)).into()
        }
        unsafe extern "system" fn Close<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandList_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Close().into()
        }
        Self { base: ID2D1Image_Vtbl::new::<Identity, Impl, OFFSET>(), Stream: Stream::<Identity, Impl, OFFSET>, Close: Close::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1CommandList as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Image as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
pub trait ID2D1CommandSink_Impl: Sized {
    fn BeginDraw(&mut self) -> ::windows::core::Result<()>;
    fn EndDraw(&mut self) -> ::windows::core::Result<()>;
    fn SetAntialiasMode(&mut self, antialiasmode: D2D1_ANTIALIAS_MODE) -> ::windows::core::Result<()>;
    fn SetTags(&mut self, tag1: u64, tag2: u64) -> ::windows::core::Result<()>;
    fn SetTextAntialiasMode(&mut self, textantialiasmode: D2D1_TEXT_ANTIALIAS_MODE) -> ::windows::core::Result<()>;
    fn SetTextRenderingParams(&mut self, textrenderingparams: &::core::option::Option<super::DirectWrite::IDWriteRenderingParams>) -> ::windows::core::Result<()>;
    fn SetTransform(&mut self, transform: *const super::super::super::Foundation::Numerics::Matrix3x2) -> ::windows::core::Result<()>;
    fn SetPrimitiveBlend(&mut self, primitiveblend: D2D1_PRIMITIVE_BLEND) -> ::windows::core::Result<()>;
    fn SetUnitMode(&mut self, unitmode: D2D1_UNIT_MODE) -> ::windows::core::Result<()>;
    fn Clear(&mut self, color: *const Common::D2D1_COLOR_F) -> ::windows::core::Result<()>;
    fn DrawGlyphRun(&mut self, baselineorigin: &Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, glyphrundescription: *const super::DirectWrite::DWRITE_GLYPH_RUN_DESCRIPTION, foregroundbrush: &::core::option::Option<ID2D1Brush>, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE) -> ::windows::core::Result<()>;
    fn DrawLine(&mut self, point0: &Common::D2D_POINT_2F, point1: &Common::D2D_POINT_2F, brush: &::core::option::Option<ID2D1Brush>, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>) -> ::windows::core::Result<()>;
    fn DrawGeometry(&mut self, geometry: &::core::option::Option<ID2D1Geometry>, brush: &::core::option::Option<ID2D1Brush>, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>) -> ::windows::core::Result<()>;
    fn DrawRectangle(&mut self, rect: *const Common::D2D_RECT_F, brush: &::core::option::Option<ID2D1Brush>, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>) -> ::windows::core::Result<()>;
    fn DrawBitmap(&mut self, bitmap: &::core::option::Option<ID2D1Bitmap>, destinationrectangle: *const Common::D2D_RECT_F, opacity: f32, interpolationmode: D2D1_INTERPOLATION_MODE, sourcerectangle: *const Common::D2D_RECT_F, perspectivetransform: *const Common::D2D_MATRIX_4X4_F) -> ::windows::core::Result<()>;
    fn DrawImage(&mut self, image: &::core::option::Option<ID2D1Image>, targetoffset: *const Common::D2D_POINT_2F, imagerectangle: *const Common::D2D_RECT_F, interpolationmode: D2D1_INTERPOLATION_MODE, compositemode: Common::D2D1_COMPOSITE_MODE) -> ::windows::core::Result<()>;
    fn DrawGdiMetafile(&mut self, gdimetafile: &::core::option::Option<ID2D1GdiMetafile>, targetoffset: *const Common::D2D_POINT_2F) -> ::windows::core::Result<()>;
    fn FillMesh(&mut self, mesh: &::core::option::Option<ID2D1Mesh>, brush: &::core::option::Option<ID2D1Brush>) -> ::windows::core::Result<()>;
    fn FillOpacityMask(&mut self, opacitymask: &::core::option::Option<ID2D1Bitmap>, brush: &::core::option::Option<ID2D1Brush>, destinationrectangle: *const Common::D2D_RECT_F, sourcerectangle: *const Common::D2D_RECT_F) -> ::windows::core::Result<()>;
    fn FillGeometry(&mut self, geometry: &::core::option::Option<ID2D1Geometry>, brush: &::core::option::Option<ID2D1Brush>, opacitybrush: &::core::option::Option<ID2D1Brush>) -> ::windows::core::Result<()>;
    fn FillRectangle(&mut self, rect: *const Common::D2D_RECT_F, brush: &::core::option::Option<ID2D1Brush>) -> ::windows::core::Result<()>;
    fn PushAxisAlignedClip(&mut self, cliprect: *const Common::D2D_RECT_F, antialiasmode: D2D1_ANTIALIAS_MODE) -> ::windows::core::Result<()>;
    fn PushLayer(&mut self, layerparameters1: *const D2D1_LAYER_PARAMETERS1, layer: &::core::option::Option<ID2D1Layer>) -> ::windows::core::Result<()>;
    fn PopAxisAlignedClip(&mut self) -> ::windows::core::Result<()>;
    fn PopLayer(&mut self) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
impl ID2D1CommandSink_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>() -> ID2D1CommandSink_Vtbl {
        unsafe extern "system" fn BeginDraw<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).BeginDraw().into()
        }
        unsafe extern "system" fn EndDraw<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).EndDraw().into()
        }
        unsafe extern "system" fn SetAntialiasMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, antialiasmode: D2D1_ANTIALIAS_MODE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetAntialiasMode(::core::mem::transmute_copy(&antialiasmode)).into()
        }
        unsafe extern "system" fn SetTags<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, tag1: u64, tag2: u64) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTags(::core::mem::transmute_copy(&tag1), ::core::mem::transmute_copy(&tag2)).into()
        }
        unsafe extern "system" fn SetTextAntialiasMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, textantialiasmode: D2D1_TEXT_ANTIALIAS_MODE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTextAntialiasMode(::core::mem::transmute_copy(&textantialiasmode)).into()
        }
        unsafe extern "system" fn SetTextRenderingParams<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, textrenderingparams: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTextRenderingParams(::core::mem::transmute(&textrenderingparams)).into()
        }
        unsafe extern "system" fn SetTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, transform: *const super::super::super::Foundation::Numerics::Matrix3x2) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTransform(::core::mem::transmute_copy(&transform)).into()
        }
        unsafe extern "system" fn SetPrimitiveBlend<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, primitiveblend: D2D1_PRIMITIVE_BLEND) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetPrimitiveBlend(::core::mem::transmute_copy(&primitiveblend)).into()
        }
        unsafe extern "system" fn SetUnitMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, unitmode: D2D1_UNIT_MODE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetUnitMode(::core::mem::transmute_copy(&unitmode)).into()
        }
        unsafe extern "system" fn Clear<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, color: *const Common::D2D1_COLOR_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Clear(::core::mem::transmute_copy(&color)).into()
        }
        unsafe extern "system" fn DrawGlyphRun<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, baselineorigin: Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, glyphrundescription: *const super::DirectWrite::DWRITE_GLYPH_RUN_DESCRIPTION, foregroundbrush: ::windows::core::RawPtr, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGlyphRun(::core::mem::transmute_copy(&baselineorigin), ::core::mem::transmute_copy(&glyphrun), ::core::mem::transmute_copy(&glyphrundescription), ::core::mem::transmute(&foregroundbrush), ::core::mem::transmute_copy(&measuringmode)).into()
        }
        unsafe extern "system" fn DrawLine<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, point0: Common::D2D_POINT_2F, point1: Common::D2D_POINT_2F, brush: ::windows::core::RawPtr, strokewidth: f32, strokestyle: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawLine(::core::mem::transmute_copy(&point0), ::core::mem::transmute_copy(&point1), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle)).into()
        }
        unsafe extern "system" fn DrawGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, geometry: ::windows::core::RawPtr, brush: ::windows::core::RawPtr, strokewidth: f32, strokestyle: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGeometry(::core::mem::transmute(&geometry), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle)).into()
        }
        unsafe extern "system" fn DrawRectangle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rect: *const Common::D2D_RECT_F, brush: ::windows::core::RawPtr, strokewidth: f32, strokestyle: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawRectangle(::core::mem::transmute_copy(&rect), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle)).into()
        }
        unsafe extern "system" fn DrawBitmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bitmap: ::windows::core::RawPtr, destinationrectangle: *const Common::D2D_RECT_F, opacity: f32, interpolationmode: D2D1_INTERPOLATION_MODE, sourcerectangle: *const Common::D2D_RECT_F, perspectivetransform: *const Common::D2D_MATRIX_4X4_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawBitmap(::core::mem::transmute(&bitmap), ::core::mem::transmute_copy(&destinationrectangle), ::core::mem::transmute_copy(&opacity), ::core::mem::transmute_copy(&interpolationmode), ::core::mem::transmute_copy(&sourcerectangle), ::core::mem::transmute_copy(&perspectivetransform)).into()
        }
        unsafe extern "system" fn DrawImage<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: ::windows::core::RawPtr, targetoffset: *const Common::D2D_POINT_2F, imagerectangle: *const Common::D2D_RECT_F, interpolationmode: D2D1_INTERPOLATION_MODE, compositemode: Common::D2D1_COMPOSITE_MODE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawImage(::core::mem::transmute(&image), ::core::mem::transmute_copy(&targetoffset), ::core::mem::transmute_copy(&imagerectangle), ::core::mem::transmute_copy(&interpolationmode), ::core::mem::transmute_copy(&compositemode)).into()
        }
        unsafe extern "system" fn DrawGdiMetafile<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gdimetafile: ::windows::core::RawPtr, targetoffset: *const Common::D2D_POINT_2F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGdiMetafile(::core::mem::transmute(&gdimetafile), ::core::mem::transmute_copy(&targetoffset)).into()
        }
        unsafe extern "system" fn FillMesh<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, mesh: ::windows::core::RawPtr, brush: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillMesh(::core::mem::transmute(&mesh), ::core::mem::transmute(&brush)).into()
        }
        unsafe extern "system" fn FillOpacityMask<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, opacitymask: ::windows::core::RawPtr, brush: ::windows::core::RawPtr, destinationrectangle: *const Common::D2D_RECT_F, sourcerectangle: *const Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillOpacityMask(::core::mem::transmute(&opacitymask), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&destinationrectangle), ::core::mem::transmute_copy(&sourcerectangle)).into()
        }
        unsafe extern "system" fn FillGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, geometry: ::windows::core::RawPtr, brush: ::windows::core::RawPtr, opacitybrush: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillGeometry(::core::mem::transmute(&geometry), ::core::mem::transmute(&brush), ::core::mem::transmute(&opacitybrush)).into()
        }
        unsafe extern "system" fn FillRectangle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rect: *const Common::D2D_RECT_F, brush: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillRectangle(::core::mem::transmute_copy(&rect), ::core::mem::transmute(&brush)).into()
        }
        unsafe extern "system" fn PushAxisAlignedClip<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, cliprect: *const Common::D2D_RECT_F, antialiasmode: D2D1_ANTIALIAS_MODE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).PushAxisAlignedClip(::core::mem::transmute_copy(&cliprect), ::core::mem::transmute_copy(&antialiasmode)).into()
        }
        unsafe extern "system" fn PushLayer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, layerparameters1: *const D2D1_LAYER_PARAMETERS1, layer: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).PushLayer(::core::mem::transmute_copy(&layerparameters1), ::core::mem::transmute(&layer)).into()
        }
        unsafe extern "system" fn PopAxisAlignedClip<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).PopAxisAlignedClip().into()
        }
        unsafe extern "system" fn PopLayer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).PopLayer().into()
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            BeginDraw: BeginDraw::<Identity, Impl, OFFSET>,
            EndDraw: EndDraw::<Identity, Impl, OFFSET>,
            SetAntialiasMode: SetAntialiasMode::<Identity, Impl, OFFSET>,
            SetTags: SetTags::<Identity, Impl, OFFSET>,
            SetTextAntialiasMode: SetTextAntialiasMode::<Identity, Impl, OFFSET>,
            SetTextRenderingParams: SetTextRenderingParams::<Identity, Impl, OFFSET>,
            SetTransform: SetTransform::<Identity, Impl, OFFSET>,
            SetPrimitiveBlend: SetPrimitiveBlend::<Identity, Impl, OFFSET>,
            SetUnitMode: SetUnitMode::<Identity, Impl, OFFSET>,
            Clear: Clear::<Identity, Impl, OFFSET>,
            DrawGlyphRun: DrawGlyphRun::<Identity, Impl, OFFSET>,
            DrawLine: DrawLine::<Identity, Impl, OFFSET>,
            DrawGeometry: DrawGeometry::<Identity, Impl, OFFSET>,
            DrawRectangle: DrawRectangle::<Identity, Impl, OFFSET>,
            DrawBitmap: DrawBitmap::<Identity, Impl, OFFSET>,
            DrawImage: DrawImage::<Identity, Impl, OFFSET>,
            DrawGdiMetafile: DrawGdiMetafile::<Identity, Impl, OFFSET>,
            FillMesh: FillMesh::<Identity, Impl, OFFSET>,
            FillOpacityMask: FillOpacityMask::<Identity, Impl, OFFSET>,
            FillGeometry: FillGeometry::<Identity, Impl, OFFSET>,
            FillRectangle: FillRectangle::<Identity, Impl, OFFSET>,
            PushAxisAlignedClip: PushAxisAlignedClip::<Identity, Impl, OFFSET>,
            PushLayer: PushLayer::<Identity, Impl, OFFSET>,
            PopAxisAlignedClip: PopAxisAlignedClip::<Identity, Impl, OFFSET>,
            PopLayer: PopLayer::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1CommandSink as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
pub trait ID2D1CommandSink1_Impl: Sized + ID2D1CommandSink_Impl {
    fn SetPrimitiveBlend1(&mut self, primitiveblend: D2D1_PRIMITIVE_BLEND) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
impl ID2D1CommandSink1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink1_Impl, const OFFSET: isize>() -> ID2D1CommandSink1_Vtbl {
        unsafe extern "system" fn SetPrimitiveBlend1<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, primitiveblend: D2D1_PRIMITIVE_BLEND) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetPrimitiveBlend1(::core::mem::transmute_copy(&primitiveblend)).into()
        }
        Self { base: ID2D1CommandSink_Vtbl::new::<Identity, Impl, OFFSET>(), SetPrimitiveBlend1: SetPrimitiveBlend1::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1CommandSink1 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
pub trait ID2D1CommandSink2_Impl: Sized + ID2D1CommandSink_Impl + ID2D1CommandSink1_Impl {
    fn DrawInk(&mut self, ink: &::core::option::Option<ID2D1Ink>, brush: &::core::option::Option<ID2D1Brush>, inkstyle: &::core::option::Option<ID2D1InkStyle>) -> ::windows::core::Result<()>;
    fn DrawGradientMesh(&mut self, gradientmesh: &::core::option::Option<ID2D1GradientMesh>) -> ::windows::core::Result<()>;
    fn DrawGdiMetafile2(&mut self, gdimetafile: &::core::option::Option<ID2D1GdiMetafile>, destinationrectangle: *const Common::D2D_RECT_F, sourcerectangle: *const Common::D2D_RECT_F) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
impl ID2D1CommandSink2_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink2_Impl, const OFFSET: isize>() -> ID2D1CommandSink2_Vtbl {
        unsafe extern "system" fn DrawInk<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, ink: ::windows::core::RawPtr, brush: ::windows::core::RawPtr, inkstyle: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawInk(::core::mem::transmute(&ink), ::core::mem::transmute(&brush), ::core::mem::transmute(&inkstyle)).into()
        }
        unsafe extern "system" fn DrawGradientMesh<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gradientmesh: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGradientMesh(::core::mem::transmute(&gradientmesh)).into()
        }
        unsafe extern "system" fn DrawGdiMetafile2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gdimetafile: ::windows::core::RawPtr, destinationrectangle: *const Common::D2D_RECT_F, sourcerectangle: *const Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGdiMetafile2(::core::mem::transmute(&gdimetafile), ::core::mem::transmute_copy(&destinationrectangle), ::core::mem::transmute_copy(&sourcerectangle)).into()
        }
        Self {
            base: ID2D1CommandSink1_Vtbl::new::<Identity, Impl, OFFSET>(),
            DrawInk: DrawInk::<Identity, Impl, OFFSET>,
            DrawGradientMesh: DrawGradientMesh::<Identity, Impl, OFFSET>,
            DrawGdiMetafile2: DrawGdiMetafile2::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1CommandSink2 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink1 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
pub trait ID2D1CommandSink3_Impl: Sized + ID2D1CommandSink_Impl + ID2D1CommandSink1_Impl + ID2D1CommandSink2_Impl {
    fn DrawSpriteBatch(&mut self, spritebatch: &::core::option::Option<ID2D1SpriteBatch>, startindex: u32, spritecount: u32, bitmap: &::core::option::Option<ID2D1Bitmap>, interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE, spriteoptions: D2D1_SPRITE_OPTIONS) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
impl ID2D1CommandSink3_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink3_Impl, const OFFSET: isize>() -> ID2D1CommandSink3_Vtbl {
        unsafe extern "system" fn DrawSpriteBatch<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink3_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, spritebatch: ::windows::core::RawPtr, startindex: u32, spritecount: u32, bitmap: ::windows::core::RawPtr, interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE, spriteoptions: D2D1_SPRITE_OPTIONS) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawSpriteBatch(::core::mem::transmute(&spritebatch), ::core::mem::transmute_copy(&startindex), ::core::mem::transmute_copy(&spritecount), ::core::mem::transmute(&bitmap), ::core::mem::transmute_copy(&interpolationmode), ::core::mem::transmute_copy(&spriteoptions)).into()
        }
        Self { base: ID2D1CommandSink2_Vtbl::new::<Identity, Impl, OFFSET>(), DrawSpriteBatch: DrawSpriteBatch::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1CommandSink3 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink1 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink2 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
pub trait ID2D1CommandSink4_Impl: Sized + ID2D1CommandSink_Impl + ID2D1CommandSink1_Impl + ID2D1CommandSink2_Impl + ID2D1CommandSink3_Impl {
    fn SetPrimitiveBlend2(&mut self, primitiveblend: D2D1_PRIMITIVE_BLEND) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
impl ID2D1CommandSink4_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink4_Impl, const OFFSET: isize>() -> ID2D1CommandSink4_Vtbl {
        unsafe extern "system" fn SetPrimitiveBlend2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, primitiveblend: D2D1_PRIMITIVE_BLEND) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetPrimitiveBlend2(::core::mem::transmute_copy(&primitiveblend)).into()
        }
        Self { base: ID2D1CommandSink3_Vtbl::new::<Identity, Impl, OFFSET>(), SetPrimitiveBlend2: SetPrimitiveBlend2::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1CommandSink4 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink1 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink2 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink3 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
pub trait ID2D1CommandSink5_Impl: Sized + ID2D1CommandSink_Impl + ID2D1CommandSink1_Impl + ID2D1CommandSink2_Impl + ID2D1CommandSink3_Impl + ID2D1CommandSink4_Impl {
    fn BlendImage(&mut self, image: &::core::option::Option<ID2D1Image>, blendmode: Common::D2D1_BLEND_MODE, targetoffset: *const Common::D2D_POINT_2F, imagerectangle: *const Common::D2D_RECT_F, interpolationmode: D2D1_INTERPOLATION_MODE) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite"))]
impl ID2D1CommandSink5_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink5_Impl, const OFFSET: isize>() -> ID2D1CommandSink5_Vtbl {
        unsafe extern "system" fn BlendImage<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1CommandSink5_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: ::windows::core::RawPtr, blendmode: Common::D2D1_BLEND_MODE, targetoffset: *const Common::D2D_POINT_2F, imagerectangle: *const Common::D2D_RECT_F, interpolationmode: D2D1_INTERPOLATION_MODE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).BlendImage(::core::mem::transmute(&image), ::core::mem::transmute_copy(&blendmode), ::core::mem::transmute_copy(&targetoffset), ::core::mem::transmute_copy(&imagerectangle), ::core::mem::transmute_copy(&interpolationmode)).into()
        }
        Self { base: ID2D1CommandSink4_Vtbl::new::<Identity, Impl, OFFSET>(), BlendImage: BlendImage::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1CommandSink5 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink1 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink2 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink3 as ::windows::core::Interface>::IID || iid == &<ID2D1CommandSink4 as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1ComputeInfo_Impl: Sized + ID2D1RenderInfo_Impl {
    fn SetComputeShaderConstantBuffer(&mut self, buffer: *const u8, buffercount: u32) -> ::windows::core::Result<()>;
    fn SetComputeShader(&mut self, shaderid: *const ::windows::core::GUID) -> ::windows::core::Result<()>;
    fn SetResourceTexture(&mut self, textureindex: u32, resourcetexture: &::core::option::Option<ID2D1ResourceTexture>) -> ::windows::core::Result<()>;
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1ComputeInfo_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ComputeInfo_Impl, const OFFSET: isize>() -> ID2D1ComputeInfo_Vtbl {
        unsafe extern "system" fn SetComputeShaderConstantBuffer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ComputeInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, buffer: *const u8, buffercount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetComputeShaderConstantBuffer(::core::mem::transmute_copy(&buffer), ::core::mem::transmute_copy(&buffercount)).into()
        }
        unsafe extern "system" fn SetComputeShader<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ComputeInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, shaderid: *const ::windows::core::GUID) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetComputeShader(::core::mem::transmute_copy(&shaderid)).into()
        }
        unsafe extern "system" fn SetResourceTexture<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ComputeInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, textureindex: u32, resourcetexture: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetResourceTexture(::core::mem::transmute_copy(&textureindex), ::core::mem::transmute(&resourcetexture)).into()
        }
        Self {
            base: ID2D1RenderInfo_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetComputeShaderConstantBuffer: SetComputeShaderConstantBuffer::<Identity, Impl, OFFSET>,
            SetComputeShader: SetComputeShader::<Identity, Impl, OFFSET>,
            SetResourceTexture: SetResourceTexture::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1ComputeInfo as ::windows::core::Interface>::IID || iid == &<ID2D1RenderInfo as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1ComputeTransform_Impl: Sized + ID2D1TransformNode_Impl + ID2D1Transform_Impl {
    fn SetComputeInfo(&mut self, computeinfo: &::core::option::Option<ID2D1ComputeInfo>) -> ::windows::core::Result<()>;
    fn CalculateThreadgroups(&mut self, outputrect: *const super::super::Foundation::RECT, dimensionx: *mut u32, dimensiony: *mut u32, dimensionz: *mut u32) -> ::windows::core::Result<()>;
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1ComputeTransform_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ComputeTransform_Impl, const OFFSET: isize>() -> ID2D1ComputeTransform_Vtbl {
        unsafe extern "system" fn SetComputeInfo<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ComputeTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, computeinfo: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetComputeInfo(::core::mem::transmute(&computeinfo)).into()
        }
        unsafe extern "system" fn CalculateThreadgroups<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ComputeTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, outputrect: *const super::super::Foundation::RECT, dimensionx: *mut u32, dimensiony: *mut u32, dimensionz: *mut u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).CalculateThreadgroups(::core::mem::transmute_copy(&outputrect), ::core::mem::transmute_copy(&dimensionx), ::core::mem::transmute_copy(&dimensiony), ::core::mem::transmute_copy(&dimensionz)).into()
        }
        Self {
            base: ID2D1Transform_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetComputeInfo: SetComputeInfo::<Identity, Impl, OFFSET>,
            CalculateThreadgroups: CalculateThreadgroups::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1ComputeTransform as ::windows::core::Interface>::IID || iid == &<ID2D1TransformNode as ::windows::core::Interface>::IID || iid == &<ID2D1Transform as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1ConcreteTransform_Impl: Sized + ID2D1TransformNode_Impl {
    fn SetOutputBuffer(&mut self, bufferprecision: D2D1_BUFFER_PRECISION, channeldepth: D2D1_CHANNEL_DEPTH) -> ::windows::core::Result<()>;
    fn SetCached(&mut self, iscached: super::super::Foundation::BOOL);
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1ConcreteTransform_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ConcreteTransform_Impl, const OFFSET: isize>() -> ID2D1ConcreteTransform_Vtbl {
        unsafe extern "system" fn SetOutputBuffer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ConcreteTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bufferprecision: D2D1_BUFFER_PRECISION, channeldepth: D2D1_CHANNEL_DEPTH) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetOutputBuffer(::core::mem::transmute_copy(&bufferprecision), ::core::mem::transmute_copy(&channeldepth)).into()
        }
        unsafe extern "system" fn SetCached<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ConcreteTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, iscached: super::super::Foundation::BOOL) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetCached(::core::mem::transmute_copy(&iscached))
        }
        Self {
            base: ID2D1TransformNode_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetOutputBuffer: SetOutputBuffer::<Identity, Impl, OFFSET>,
            SetCached: SetCached::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1ConcreteTransform as ::windows::core::Interface>::IID || iid == &<ID2D1TransformNode as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Gdi", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1DCRenderTarget_Impl: Sized + ID2D1Resource_Impl + ID2D1RenderTarget_Impl {
    fn BindDC(&mut self, hdc: super::Gdi::HDC, psubrect: *const super::super::Foundation::RECT) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Gdi", feature = "Win32_Graphics_Imaging"))]
impl ID2D1DCRenderTarget_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DCRenderTarget_Impl, const OFFSET: isize>() -> ID2D1DCRenderTarget_Vtbl {
        unsafe extern "system" fn BindDC<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DCRenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, hdc: super::Gdi::HDC, psubrect: *const super::super::Foundation::RECT) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).BindDC(::core::mem::transmute_copy(&hdc), ::core::mem::transmute_copy(&psubrect)).into()
        }
        Self { base: ID2D1RenderTarget_Vtbl::new::<Identity, Impl, OFFSET>(), BindDC: BindDC::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DCRenderTarget as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
pub trait ID2D1Device_Impl: Sized + ID2D1Resource_Impl {
    fn CreateDeviceContext(&mut self, options: D2D1_DEVICE_CONTEXT_OPTIONS) -> ::windows::core::Result<ID2D1DeviceContext>;
    fn CreatePrintControl(&mut self, wicfactory: &::core::option::Option<super::Imaging::IWICImagingFactory>, documenttarget: &::core::option::Option<super::super::Storage::Xps::Printing::IPrintDocumentPackageTarget>, printcontrolproperties: *const D2D1_PRINT_CONTROL_PROPERTIES) -> ::windows::core::Result<ID2D1PrintControl>;
    fn SetMaximumTextureMemory(&mut self, maximuminbytes: u64);
    fn GetMaximumTextureMemory(&mut self) -> u64;
    fn ClearResources(&mut self, millisecondssinceuse: u32);
}
#[cfg(all(feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
impl ID2D1Device_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device_Impl, const OFFSET: isize>() -> ID2D1Device_Vtbl {
        unsafe extern "system" fn CreateDeviceContext<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, options: D2D1_DEVICE_CONTEXT_OPTIONS, devicecontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDeviceContext(::core::mem::transmute_copy(&options)) {
                ::core::result::Result::Ok(ok__) => {
                    *devicecontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreatePrintControl<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, wicfactory: ::windows::core::RawPtr, documenttarget: ::windows::core::RawPtr, printcontrolproperties: *const D2D1_PRINT_CONTROL_PROPERTIES, printcontrol: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreatePrintControl(::core::mem::transmute(&wicfactory), ::core::mem::transmute(&documenttarget), ::core::mem::transmute_copy(&printcontrolproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *printcontrol = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn SetMaximumTextureMemory<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, maximuminbytes: u64) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetMaximumTextureMemory(::core::mem::transmute_copy(&maximuminbytes))
        }
        unsafe extern "system" fn GetMaximumTextureMemory<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u64 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetMaximumTextureMemory()
        }
        unsafe extern "system" fn ClearResources<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, millisecondssinceuse: u32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ClearResources(::core::mem::transmute_copy(&millisecondssinceuse))
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateDeviceContext: CreateDeviceContext::<Identity, Impl, OFFSET>,
            CreatePrintControl: CreatePrintControl::<Identity, Impl, OFFSET>,
            SetMaximumTextureMemory: SetMaximumTextureMemory::<Identity, Impl, OFFSET>,
            GetMaximumTextureMemory: GetMaximumTextureMemory::<Identity, Impl, OFFSET>,
            ClearResources: ClearResources::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Device as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
pub trait ID2D1Device1_Impl: Sized + ID2D1Resource_Impl + ID2D1Device_Impl {
    fn GetRenderingPriority(&mut self) -> D2D1_RENDERING_PRIORITY;
    fn SetRenderingPriority(&mut self, renderingpriority: D2D1_RENDERING_PRIORITY);
    fn CreateDeviceContext2(&mut self, options: D2D1_DEVICE_CONTEXT_OPTIONS) -> ::windows::core::Result<ID2D1DeviceContext1>;
}
#[cfg(all(feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
impl ID2D1Device1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device1_Impl, const OFFSET: isize>() -> ID2D1Device1_Vtbl {
        unsafe extern "system" fn GetRenderingPriority<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_RENDERING_PRIORITY {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetRenderingPriority()
        }
        unsafe extern "system" fn SetRenderingPriority<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, renderingpriority: D2D1_RENDERING_PRIORITY) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetRenderingPriority(::core::mem::transmute_copy(&renderingpriority))
        }
        unsafe extern "system" fn CreateDeviceContext2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, options: D2D1_DEVICE_CONTEXT_OPTIONS, devicecontext1: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDeviceContext2(::core::mem::transmute_copy(&options)) {
                ::core::result::Result::Ok(ok__) => {
                    *devicecontext1 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1Device_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetRenderingPriority: GetRenderingPriority::<Identity, Impl, OFFSET>,
            SetRenderingPriority: SetRenderingPriority::<Identity, Impl, OFFSET>,
            CreateDeviceContext2: CreateDeviceContext2::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Device1 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Device as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
pub trait ID2D1Device2_Impl: Sized + ID2D1Resource_Impl + ID2D1Device_Impl + ID2D1Device1_Impl {
    fn CreateDeviceContext3(&mut self, options: D2D1_DEVICE_CONTEXT_OPTIONS) -> ::windows::core::Result<ID2D1DeviceContext2>;
    fn FlushDeviceContexts(&mut self, bitmap: &::core::option::Option<ID2D1Bitmap>);
    fn GetDxgiDevice(&mut self) -> ::windows::core::Result<super::Dxgi::IDXGIDevice>;
}
#[cfg(all(feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
impl ID2D1Device2_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device2_Impl, const OFFSET: isize>() -> ID2D1Device2_Vtbl {
        unsafe extern "system" fn CreateDeviceContext3<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, options: D2D1_DEVICE_CONTEXT_OPTIONS, devicecontext2: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDeviceContext3(::core::mem::transmute_copy(&options)) {
                ::core::result::Result::Ok(ok__) => {
                    *devicecontext2 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn FlushDeviceContexts<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bitmap: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FlushDeviceContexts(::core::mem::transmute(&bitmap))
        }
        unsafe extern "system" fn GetDxgiDevice<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dxgidevice: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetDxgiDevice() {
                ::core::result::Result::Ok(ok__) => {
                    *dxgidevice = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1Device1_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateDeviceContext3: CreateDeviceContext3::<Identity, Impl, OFFSET>,
            FlushDeviceContexts: FlushDeviceContexts::<Identity, Impl, OFFSET>,
            GetDxgiDevice: GetDxgiDevice::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Device2 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Device as ::windows::core::Interface>::IID || iid == &<ID2D1Device1 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
pub trait ID2D1Device3_Impl: Sized + ID2D1Resource_Impl + ID2D1Device_Impl + ID2D1Device1_Impl + ID2D1Device2_Impl {
    fn CreateDeviceContext4(&mut self, options: D2D1_DEVICE_CONTEXT_OPTIONS) -> ::windows::core::Result<ID2D1DeviceContext3>;
}
#[cfg(all(feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
impl ID2D1Device3_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device3_Impl, const OFFSET: isize>() -> ID2D1Device3_Vtbl {
        unsafe extern "system" fn CreateDeviceContext4<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device3_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, options: D2D1_DEVICE_CONTEXT_OPTIONS, devicecontext3: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDeviceContext4(::core::mem::transmute_copy(&options)) {
                ::core::result::Result::Ok(ok__) => {
                    *devicecontext3 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1Device2_Vtbl::new::<Identity, Impl, OFFSET>(), CreateDeviceContext4: CreateDeviceContext4::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Device3 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Device as ::windows::core::Interface>::IID || iid == &<ID2D1Device1 as ::windows::core::Interface>::IID || iid == &<ID2D1Device2 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
pub trait ID2D1Device4_Impl: Sized + ID2D1Resource_Impl + ID2D1Device_Impl + ID2D1Device1_Impl + ID2D1Device2_Impl + ID2D1Device3_Impl {
    fn CreateDeviceContext5(&mut self, options: D2D1_DEVICE_CONTEXT_OPTIONS) -> ::windows::core::Result<ID2D1DeviceContext4>;
    fn SetMaximumColorGlyphCacheMemory(&mut self, maximuminbytes: u64);
    fn GetMaximumColorGlyphCacheMemory(&mut self) -> u64;
}
#[cfg(all(feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
impl ID2D1Device4_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device4_Impl, const OFFSET: isize>() -> ID2D1Device4_Vtbl {
        unsafe extern "system" fn CreateDeviceContext5<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, options: D2D1_DEVICE_CONTEXT_OPTIONS, devicecontext4: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDeviceContext5(::core::mem::transmute_copy(&options)) {
                ::core::result::Result::Ok(ok__) => {
                    *devicecontext4 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn SetMaximumColorGlyphCacheMemory<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, maximuminbytes: u64) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetMaximumColorGlyphCacheMemory(::core::mem::transmute_copy(&maximuminbytes))
        }
        unsafe extern "system" fn GetMaximumColorGlyphCacheMemory<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u64 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetMaximumColorGlyphCacheMemory()
        }
        Self {
            base: ID2D1Device3_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateDeviceContext5: CreateDeviceContext5::<Identity, Impl, OFFSET>,
            SetMaximumColorGlyphCacheMemory: SetMaximumColorGlyphCacheMemory::<Identity, Impl, OFFSET>,
            GetMaximumColorGlyphCacheMemory: GetMaximumColorGlyphCacheMemory::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Device4 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Device as ::windows::core::Interface>::IID || iid == &<ID2D1Device1 as ::windows::core::Interface>::IID || iid == &<ID2D1Device2 as ::windows::core::Interface>::IID || iid == &<ID2D1Device3 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
pub trait ID2D1Device5_Impl: Sized + ID2D1Resource_Impl + ID2D1Device_Impl + ID2D1Device1_Impl + ID2D1Device2_Impl + ID2D1Device3_Impl + ID2D1Device4_Impl {
    fn CreateDeviceContext6(&mut self, options: D2D1_DEVICE_CONTEXT_OPTIONS) -> ::windows::core::Result<ID2D1DeviceContext5>;
}
#[cfg(all(feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
impl ID2D1Device5_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device5_Impl, const OFFSET: isize>() -> ID2D1Device5_Vtbl {
        unsafe extern "system" fn CreateDeviceContext6<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device5_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, options: D2D1_DEVICE_CONTEXT_OPTIONS, devicecontext5: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDeviceContext6(::core::mem::transmute_copy(&options)) {
                ::core::result::Result::Ok(ok__) => {
                    *devicecontext5 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1Device4_Vtbl::new::<Identity, Impl, OFFSET>(), CreateDeviceContext6: CreateDeviceContext6::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Device5 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Device as ::windows::core::Interface>::IID || iid == &<ID2D1Device1 as ::windows::core::Interface>::IID || iid == &<ID2D1Device2 as ::windows::core::Interface>::IID || iid == &<ID2D1Device3 as ::windows::core::Interface>::IID || iid == &<ID2D1Device4 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
pub trait ID2D1Device6_Impl: Sized + ID2D1Resource_Impl + ID2D1Device_Impl + ID2D1Device1_Impl + ID2D1Device2_Impl + ID2D1Device3_Impl + ID2D1Device4_Impl + ID2D1Device5_Impl {
    fn CreateDeviceContext7(&mut self, options: D2D1_DEVICE_CONTEXT_OPTIONS) -> ::windows::core::Result<ID2D1DeviceContext6>;
}
#[cfg(all(feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Imaging", feature = "Win32_Storage_Xps_Printing"))]
impl ID2D1Device6_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device6_Impl, const OFFSET: isize>() -> ID2D1Device6_Vtbl {
        unsafe extern "system" fn CreateDeviceContext7<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Device6_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, options: D2D1_DEVICE_CONTEXT_OPTIONS, devicecontext6: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDeviceContext7(::core::mem::transmute_copy(&options)) {
                ::core::result::Result::Ok(ok__) => {
                    *devicecontext6 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1Device5_Vtbl::new::<Identity, Impl, OFFSET>(), CreateDeviceContext7: CreateDeviceContext7::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Device6 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Device as ::windows::core::Interface>::IID || iid == &<ID2D1Device1 as ::windows::core::Interface>::IID || iid == &<ID2D1Device2 as ::windows::core::Interface>::IID || iid == &<ID2D1Device3 as ::windows::core::Interface>::IID || iid == &<ID2D1Device4 as ::windows::core::Interface>::IID || iid == &<ID2D1Device5 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1DeviceContext_Impl: Sized + ID2D1Resource_Impl + ID2D1RenderTarget_Impl {
    fn CreateBitmap2(&mut self, size: &Common::D2D_SIZE_U, sourcedata: *const ::core::ffi::c_void, pitch: u32, bitmapproperties: *const D2D1_BITMAP_PROPERTIES1) -> ::windows::core::Result<ID2D1Bitmap1>;
    fn CreateBitmapFromWicBitmap2(&mut self, wicbitmapsource: &::core::option::Option<super::Imaging::IWICBitmapSource>, bitmapproperties: *const D2D1_BITMAP_PROPERTIES1) -> ::windows::core::Result<ID2D1Bitmap1>;
    fn CreateColorContext(&mut self, space: D2D1_COLOR_SPACE, profile: *const u8, profilesize: u32) -> ::windows::core::Result<ID2D1ColorContext>;
    fn CreateColorContextFromFilename(&mut self, filename: super::super::Foundation::PWSTR) -> ::windows::core::Result<ID2D1ColorContext>;
    fn CreateColorContextFromWicColorContext(&mut self, wiccolorcontext: &::core::option::Option<super::Imaging::IWICColorContext>) -> ::windows::core::Result<ID2D1ColorContext>;
    fn CreateBitmapFromDxgiSurface(&mut self, surface: &::core::option::Option<super::Dxgi::IDXGISurface>, bitmapproperties: *const D2D1_BITMAP_PROPERTIES1) -> ::windows::core::Result<ID2D1Bitmap1>;
    fn CreateEffect(&mut self, effectid: *const ::windows::core::GUID) -> ::windows::core::Result<ID2D1Effect>;
    fn CreateGradientStopCollection2(&mut self, straightalphagradientstops: *const D2D1_GRADIENT_STOP, straightalphagradientstopscount: u32, preinterpolationspace: D2D1_COLOR_SPACE, postinterpolationspace: D2D1_COLOR_SPACE, bufferprecision: D2D1_BUFFER_PRECISION, extendmode: D2D1_EXTEND_MODE, colorinterpolationmode: D2D1_COLOR_INTERPOLATION_MODE) -> ::windows::core::Result<ID2D1GradientStopCollection1>;
    fn CreateImageBrush(&mut self, image: &::core::option::Option<ID2D1Image>, imagebrushproperties: *const D2D1_IMAGE_BRUSH_PROPERTIES, brushproperties: *const D2D1_BRUSH_PROPERTIES) -> ::windows::core::Result<ID2D1ImageBrush>;
    fn CreateBitmapBrush2(&mut self, bitmap: &::core::option::Option<ID2D1Bitmap>, bitmapbrushproperties: *const D2D1_BITMAP_BRUSH_PROPERTIES1, brushproperties: *const D2D1_BRUSH_PROPERTIES) -> ::windows::core::Result<ID2D1BitmapBrush1>;
    fn CreateCommandList(&mut self) -> ::windows::core::Result<ID2D1CommandList>;
    fn IsDxgiFormatSupported(&mut self, format: super::Dxgi::Common::DXGI_FORMAT) -> super::super::Foundation::BOOL;
    fn IsBufferPrecisionSupported(&mut self, bufferprecision: D2D1_BUFFER_PRECISION) -> super::super::Foundation::BOOL;
    fn GetImageLocalBounds(&mut self, image: &::core::option::Option<ID2D1Image>) -> ::windows::core::Result<Common::D2D_RECT_F>;
    fn GetImageWorldBounds(&mut self, image: &::core::option::Option<ID2D1Image>) -> ::windows::core::Result<Common::D2D_RECT_F>;
    fn GetGlyphRunWorldBounds(&mut self, baselineorigin: &Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE) -> ::windows::core::Result<Common::D2D_RECT_F>;
    fn GetDevice(&mut self, device: *mut ::core::option::Option<ID2D1Device>);
    fn SetTarget(&mut self, image: &::core::option::Option<ID2D1Image>);
    fn GetTarget(&mut self, image: *mut ::core::option::Option<ID2D1Image>);
    fn SetRenderingControls(&mut self, renderingcontrols: *const D2D1_RENDERING_CONTROLS);
    fn GetRenderingControls(&mut self, renderingcontrols: *mut D2D1_RENDERING_CONTROLS);
    fn SetPrimitiveBlend(&mut self, primitiveblend: D2D1_PRIMITIVE_BLEND);
    fn GetPrimitiveBlend(&mut self) -> D2D1_PRIMITIVE_BLEND;
    fn SetUnitMode(&mut self, unitmode: D2D1_UNIT_MODE);
    fn GetUnitMode(&mut self) -> D2D1_UNIT_MODE;
    fn DrawGlyphRun2(&mut self, baselineorigin: &Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, glyphrundescription: *const super::DirectWrite::DWRITE_GLYPH_RUN_DESCRIPTION, foregroundbrush: &::core::option::Option<ID2D1Brush>, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE);
    fn DrawImage(&mut self, image: &::core::option::Option<ID2D1Image>, targetoffset: *const Common::D2D_POINT_2F, imagerectangle: *const Common::D2D_RECT_F, interpolationmode: D2D1_INTERPOLATION_MODE, compositemode: Common::D2D1_COMPOSITE_MODE);
    fn DrawGdiMetafile(&mut self, gdimetafile: &::core::option::Option<ID2D1GdiMetafile>, targetoffset: *const Common::D2D_POINT_2F);
    fn DrawBitmap2(&mut self, bitmap: &::core::option::Option<ID2D1Bitmap>, destinationrectangle: *const Common::D2D_RECT_F, opacity: f32, interpolationmode: D2D1_INTERPOLATION_MODE, sourcerectangle: *const Common::D2D_RECT_F, perspectivetransform: *const Common::D2D_MATRIX_4X4_F);
    fn PushLayer2(&mut self, layerparameters: *const D2D1_LAYER_PARAMETERS1, layer: &::core::option::Option<ID2D1Layer>);
    fn InvalidateEffectInputRectangle(&mut self, effect: &::core::option::Option<ID2D1Effect>, input: u32, inputrectangle: *const Common::D2D_RECT_F) -> ::windows::core::Result<()>;
    fn GetEffectInvalidRectangleCount(&mut self, effect: &::core::option::Option<ID2D1Effect>) -> ::windows::core::Result<u32>;
    fn GetEffectInvalidRectangles(&mut self, effect: &::core::option::Option<ID2D1Effect>, rectangles: *mut Common::D2D_RECT_F, rectanglescount: u32) -> ::windows::core::Result<()>;
    fn GetEffectRequiredInputRectangles(&mut self, rendereffect: &::core::option::Option<ID2D1Effect>, renderimagerectangle: *const Common::D2D_RECT_F, inputdescriptions: *const D2D1_EFFECT_INPUT_DESCRIPTION, requiredinputrects: *mut Common::D2D_RECT_F, inputcount: u32) -> ::windows::core::Result<()>;
    fn FillOpacityMask2(&mut self, opacitymask: &::core::option::Option<ID2D1Bitmap>, brush: &::core::option::Option<ID2D1Brush>, destinationrectangle: *const Common::D2D_RECT_F, sourcerectangle: *const Common::D2D_RECT_F);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1DeviceContext_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>() -> ID2D1DeviceContext_Vtbl {
        unsafe extern "system" fn CreateBitmap2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, size: Common::D2D_SIZE_U, sourcedata: *const ::core::ffi::c_void, pitch: u32, bitmapproperties: *const D2D1_BITMAP_PROPERTIES1, bitmap: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateBitmap2(::core::mem::transmute_copy(&size), ::core::mem::transmute_copy(&sourcedata), ::core::mem::transmute_copy(&pitch), ::core::mem::transmute_copy(&bitmapproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *bitmap = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateBitmapFromWicBitmap2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, wicbitmapsource: ::windows::core::RawPtr, bitmapproperties: *const D2D1_BITMAP_PROPERTIES1, bitmap: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateBitmapFromWicBitmap2(::core::mem::transmute(&wicbitmapsource), ::core::mem::transmute_copy(&bitmapproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *bitmap = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateColorContext<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, space: D2D1_COLOR_SPACE, profile: *const u8, profilesize: u32, colorcontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateColorContext(::core::mem::transmute_copy(&space), ::core::mem::transmute_copy(&profile), ::core::mem::transmute_copy(&profilesize)) {
                ::core::result::Result::Ok(ok__) => {
                    *colorcontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateColorContextFromFilename<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, filename: super::super::Foundation::PWSTR, colorcontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateColorContextFromFilename(::core::mem::transmute_copy(&filename)) {
                ::core::result::Result::Ok(ok__) => {
                    *colorcontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateColorContextFromWicColorContext<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, wiccolorcontext: ::windows::core::RawPtr, colorcontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateColorContextFromWicColorContext(::core::mem::transmute(&wiccolorcontext)) {
                ::core::result::Result::Ok(ok__) => {
                    *colorcontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateBitmapFromDxgiSurface<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, surface: ::windows::core::RawPtr, bitmapproperties: *const D2D1_BITMAP_PROPERTIES1, bitmap: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateBitmapFromDxgiSurface(::core::mem::transmute(&surface), ::core::mem::transmute_copy(&bitmapproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *bitmap = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateEffect<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, effectid: *const ::windows::core::GUID, effect: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateEffect(::core::mem::transmute_copy(&effectid)) {
                ::core::result::Result::Ok(ok__) => {
                    *effect = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateGradientStopCollection2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, straightalphagradientstops: *const D2D1_GRADIENT_STOP, straightalphagradientstopscount: u32, preinterpolationspace: D2D1_COLOR_SPACE, postinterpolationspace: D2D1_COLOR_SPACE, bufferprecision: D2D1_BUFFER_PRECISION, extendmode: D2D1_EXTEND_MODE, colorinterpolationmode: D2D1_COLOR_INTERPOLATION_MODE, gradientstopcollection1: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateGradientStopCollection2(::core::mem::transmute_copy(&straightalphagradientstops), ::core::mem::transmute_copy(&straightalphagradientstopscount), ::core::mem::transmute_copy(&preinterpolationspace), ::core::mem::transmute_copy(&postinterpolationspace), ::core::mem::transmute_copy(&bufferprecision), ::core::mem::transmute_copy(&extendmode), ::core::mem::transmute_copy(&colorinterpolationmode)) {
                ::core::result::Result::Ok(ok__) => {
                    *gradientstopcollection1 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateImageBrush<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: ::windows::core::RawPtr, imagebrushproperties: *const D2D1_IMAGE_BRUSH_PROPERTIES, brushproperties: *const D2D1_BRUSH_PROPERTIES, imagebrush: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateImageBrush(::core::mem::transmute(&image), ::core::mem::transmute_copy(&imagebrushproperties), ::core::mem::transmute_copy(&brushproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *imagebrush = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateBitmapBrush2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bitmap: ::windows::core::RawPtr, bitmapbrushproperties: *const D2D1_BITMAP_BRUSH_PROPERTIES1, brushproperties: *const D2D1_BRUSH_PROPERTIES, bitmapbrush: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateBitmapBrush2(::core::mem::transmute(&bitmap), ::core::mem::transmute_copy(&bitmapbrushproperties), ::core::mem::transmute_copy(&brushproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *bitmapbrush = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateCommandList<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, commandlist: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateCommandList() {
                ::core::result::Result::Ok(ok__) => {
                    *commandlist = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn IsDxgiFormatSupported<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, format: super::Dxgi::Common::DXGI_FORMAT) -> super::super::Foundation::BOOL {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).IsDxgiFormatSupported(::core::mem::transmute_copy(&format))
        }
        unsafe extern "system" fn IsBufferPrecisionSupported<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bufferprecision: D2D1_BUFFER_PRECISION) -> super::super::Foundation::BOOL {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).IsBufferPrecisionSupported(::core::mem::transmute_copy(&bufferprecision))
        }
        unsafe extern "system" fn GetImageLocalBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: ::windows::core::RawPtr, localbounds: *mut Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetImageLocalBounds(::core::mem::transmute(&image)) {
                ::core::result::Result::Ok(ok__) => {
                    *localbounds = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn GetImageWorldBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: ::windows::core::RawPtr, worldbounds: *mut Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetImageWorldBounds(::core::mem::transmute(&image)) {
                ::core::result::Result::Ok(ok__) => {
                    *worldbounds = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn GetGlyphRunWorldBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, baselineorigin: Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE, bounds: *mut Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetGlyphRunWorldBounds(::core::mem::transmute_copy(&baselineorigin), ::core::mem::transmute_copy(&glyphrun), ::core::mem::transmute_copy(&measuringmode)) {
                ::core::result::Result::Ok(ok__) => {
                    *bounds = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn GetDevice<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, device: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDevice(::core::mem::transmute_copy(&device))
        }
        unsafe extern "system" fn SetTarget<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTarget(::core::mem::transmute(&image))
        }
        unsafe extern "system" fn GetTarget<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTarget(::core::mem::transmute_copy(&image))
        }
        unsafe extern "system" fn SetRenderingControls<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, renderingcontrols: *const D2D1_RENDERING_CONTROLS) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetRenderingControls(::core::mem::transmute_copy(&renderingcontrols))
        }
        unsafe extern "system" fn GetRenderingControls<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, renderingcontrols: *mut D2D1_RENDERING_CONTROLS) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetRenderingControls(::core::mem::transmute_copy(&renderingcontrols))
        }
        unsafe extern "system" fn SetPrimitiveBlend<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, primitiveblend: D2D1_PRIMITIVE_BLEND) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetPrimitiveBlend(::core::mem::transmute_copy(&primitiveblend))
        }
        unsafe extern "system" fn GetPrimitiveBlend<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_PRIMITIVE_BLEND {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPrimitiveBlend()
        }
        unsafe extern "system" fn SetUnitMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, unitmode: D2D1_UNIT_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetUnitMode(::core::mem::transmute_copy(&unitmode))
        }
        unsafe extern "system" fn GetUnitMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_UNIT_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetUnitMode()
        }
        unsafe extern "system" fn DrawGlyphRun2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, baselineorigin: Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, glyphrundescription: *const super::DirectWrite::DWRITE_GLYPH_RUN_DESCRIPTION, foregroundbrush: ::windows::core::RawPtr, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGlyphRun2(::core::mem::transmute_copy(&baselineorigin), ::core::mem::transmute_copy(&glyphrun), ::core::mem::transmute_copy(&glyphrundescription), ::core::mem::transmute(&foregroundbrush), ::core::mem::transmute_copy(&measuringmode))
        }
        unsafe extern "system" fn DrawImage<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: ::windows::core::RawPtr, targetoffset: *const Common::D2D_POINT_2F, imagerectangle: *const Common::D2D_RECT_F, interpolationmode: D2D1_INTERPOLATION_MODE, compositemode: Common::D2D1_COMPOSITE_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawImage(::core::mem::transmute(&image), ::core::mem::transmute_copy(&targetoffset), ::core::mem::transmute_copy(&imagerectangle), ::core::mem::transmute_copy(&interpolationmode), ::core::mem::transmute_copy(&compositemode))
        }
        unsafe extern "system" fn DrawGdiMetafile<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gdimetafile: ::windows::core::RawPtr, targetoffset: *const Common::D2D_POINT_2F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGdiMetafile(::core::mem::transmute(&gdimetafile), ::core::mem::transmute_copy(&targetoffset))
        }
        unsafe extern "system" fn DrawBitmap2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bitmap: ::windows::core::RawPtr, destinationrectangle: *const Common::D2D_RECT_F, opacity: f32, interpolationmode: D2D1_INTERPOLATION_MODE, sourcerectangle: *const Common::D2D_RECT_F, perspectivetransform: *const Common::D2D_MATRIX_4X4_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawBitmap2(::core::mem::transmute(&bitmap), ::core::mem::transmute_copy(&destinationrectangle), ::core::mem::transmute_copy(&opacity), ::core::mem::transmute_copy(&interpolationmode), ::core::mem::transmute_copy(&sourcerectangle), ::core::mem::transmute_copy(&perspectivetransform))
        }
        unsafe extern "system" fn PushLayer2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, layerparameters: *const D2D1_LAYER_PARAMETERS1, layer: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).PushLayer2(::core::mem::transmute_copy(&layerparameters), ::core::mem::transmute(&layer))
        }
        unsafe extern "system" fn InvalidateEffectInputRectangle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, effect: ::windows::core::RawPtr, input: u32, inputrectangle: *const Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).InvalidateEffectInputRectangle(::core::mem::transmute(&effect), ::core::mem::transmute_copy(&input), ::core::mem::transmute_copy(&inputrectangle)).into()
        }
        unsafe extern "system" fn GetEffectInvalidRectangleCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, effect: ::windows::core::RawPtr, rectanglecount: *mut u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetEffectInvalidRectangleCount(::core::mem::transmute(&effect)) {
                ::core::result::Result::Ok(ok__) => {
                    *rectanglecount = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn GetEffectInvalidRectangles<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, effect: ::windows::core::RawPtr, rectangles: *mut Common::D2D_RECT_F, rectanglescount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetEffectInvalidRectangles(::core::mem::transmute(&effect), ::core::mem::transmute_copy(&rectangles), ::core::mem::transmute_copy(&rectanglescount)).into()
        }
        unsafe extern "system" fn GetEffectRequiredInputRectangles<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rendereffect: ::windows::core::RawPtr, renderimagerectangle: *const Common::D2D_RECT_F, inputdescriptions: *const D2D1_EFFECT_INPUT_DESCRIPTION, requiredinputrects: *mut Common::D2D_RECT_F, inputcount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetEffectRequiredInputRectangles(::core::mem::transmute(&rendereffect), ::core::mem::transmute_copy(&renderimagerectangle), ::core::mem::transmute_copy(&inputdescriptions), ::core::mem::transmute_copy(&requiredinputrects), ::core::mem::transmute_copy(&inputcount)).into()
        }
        unsafe extern "system" fn FillOpacityMask2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, opacitymask: ::windows::core::RawPtr, brush: ::windows::core::RawPtr, destinationrectangle: *const Common::D2D_RECT_F, sourcerectangle: *const Common::D2D_RECT_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillOpacityMask2(::core::mem::transmute(&opacitymask), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&destinationrectangle), ::core::mem::transmute_copy(&sourcerectangle))
        }
        Self {
            base: ID2D1RenderTarget_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateBitmap2: CreateBitmap2::<Identity, Impl, OFFSET>,
            CreateBitmapFromWicBitmap2: CreateBitmapFromWicBitmap2::<Identity, Impl, OFFSET>,
            CreateColorContext: CreateColorContext::<Identity, Impl, OFFSET>,
            CreateColorContextFromFilename: CreateColorContextFromFilename::<Identity, Impl, OFFSET>,
            CreateColorContextFromWicColorContext: CreateColorContextFromWicColorContext::<Identity, Impl, OFFSET>,
            CreateBitmapFromDxgiSurface: CreateBitmapFromDxgiSurface::<Identity, Impl, OFFSET>,
            CreateEffect: CreateEffect::<Identity, Impl, OFFSET>,
            CreateGradientStopCollection2: CreateGradientStopCollection2::<Identity, Impl, OFFSET>,
            CreateImageBrush: CreateImageBrush::<Identity, Impl, OFFSET>,
            CreateBitmapBrush2: CreateBitmapBrush2::<Identity, Impl, OFFSET>,
            CreateCommandList: CreateCommandList::<Identity, Impl, OFFSET>,
            IsDxgiFormatSupported: IsDxgiFormatSupported::<Identity, Impl, OFFSET>,
            IsBufferPrecisionSupported: IsBufferPrecisionSupported::<Identity, Impl, OFFSET>,
            GetImageLocalBounds: GetImageLocalBounds::<Identity, Impl, OFFSET>,
            GetImageWorldBounds: GetImageWorldBounds::<Identity, Impl, OFFSET>,
            GetGlyphRunWorldBounds: GetGlyphRunWorldBounds::<Identity, Impl, OFFSET>,
            GetDevice: GetDevice::<Identity, Impl, OFFSET>,
            SetTarget: SetTarget::<Identity, Impl, OFFSET>,
            GetTarget: GetTarget::<Identity, Impl, OFFSET>,
            SetRenderingControls: SetRenderingControls::<Identity, Impl, OFFSET>,
            GetRenderingControls: GetRenderingControls::<Identity, Impl, OFFSET>,
            SetPrimitiveBlend: SetPrimitiveBlend::<Identity, Impl, OFFSET>,
            GetPrimitiveBlend: GetPrimitiveBlend::<Identity, Impl, OFFSET>,
            SetUnitMode: SetUnitMode::<Identity, Impl, OFFSET>,
            GetUnitMode: GetUnitMode::<Identity, Impl, OFFSET>,
            DrawGlyphRun2: DrawGlyphRun2::<Identity, Impl, OFFSET>,
            DrawImage: DrawImage::<Identity, Impl, OFFSET>,
            DrawGdiMetafile: DrawGdiMetafile::<Identity, Impl, OFFSET>,
            DrawBitmap2: DrawBitmap2::<Identity, Impl, OFFSET>,
            PushLayer2: PushLayer2::<Identity, Impl, OFFSET>,
            InvalidateEffectInputRectangle: InvalidateEffectInputRectangle::<Identity, Impl, OFFSET>,
            GetEffectInvalidRectangleCount: GetEffectInvalidRectangleCount::<Identity, Impl, OFFSET>,
            GetEffectInvalidRectangles: GetEffectInvalidRectangles::<Identity, Impl, OFFSET>,
            GetEffectRequiredInputRectangles: GetEffectRequiredInputRectangles::<Identity, Impl, OFFSET>,
            FillOpacityMask2: FillOpacityMask2::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DeviceContext as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1DeviceContext1_Impl: Sized + ID2D1Resource_Impl + ID2D1RenderTarget_Impl + ID2D1DeviceContext_Impl {
    fn CreateFilledGeometryRealization(&mut self, geometry: &::core::option::Option<ID2D1Geometry>, flatteningtolerance: f32) -> ::windows::core::Result<ID2D1GeometryRealization>;
    fn CreateStrokedGeometryRealization(&mut self, geometry: &::core::option::Option<ID2D1Geometry>, flatteningtolerance: f32, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>) -> ::windows::core::Result<ID2D1GeometryRealization>;
    fn DrawGeometryRealization(&mut self, geometryrealization: &::core::option::Option<ID2D1GeometryRealization>, brush: &::core::option::Option<ID2D1Brush>);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1DeviceContext1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext1_Impl, const OFFSET: isize>() -> ID2D1DeviceContext1_Vtbl {
        unsafe extern "system" fn CreateFilledGeometryRealization<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, geometry: ::windows::core::RawPtr, flatteningtolerance: f32, geometryrealization: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateFilledGeometryRealization(::core::mem::transmute(&geometry), ::core::mem::transmute_copy(&flatteningtolerance)) {
                ::core::result::Result::Ok(ok__) => {
                    *geometryrealization = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateStrokedGeometryRealization<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, geometry: ::windows::core::RawPtr, flatteningtolerance: f32, strokewidth: f32, strokestyle: ::windows::core::RawPtr, geometryrealization: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateStrokedGeometryRealization(::core::mem::transmute(&geometry), ::core::mem::transmute_copy(&flatteningtolerance), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle)) {
                ::core::result::Result::Ok(ok__) => {
                    *geometryrealization = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn DrawGeometryRealization<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, geometryrealization: ::windows::core::RawPtr, brush: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGeometryRealization(::core::mem::transmute(&geometryrealization), ::core::mem::transmute(&brush))
        }
        Self {
            base: ID2D1DeviceContext_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateFilledGeometryRealization: CreateFilledGeometryRealization::<Identity, Impl, OFFSET>,
            CreateStrokedGeometryRealization: CreateStrokedGeometryRealization::<Identity, Impl, OFFSET>,
            DrawGeometryRealization: DrawGeometryRealization::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DeviceContext1 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1DeviceContext2_Impl: Sized + ID2D1Resource_Impl + ID2D1RenderTarget_Impl + ID2D1DeviceContext_Impl + ID2D1DeviceContext1_Impl {
    fn CreateInk(&mut self, startpoint: *const D2D1_INK_POINT) -> ::windows::core::Result<ID2D1Ink>;
    fn CreateInkStyle(&mut self, inkstyleproperties: *const D2D1_INK_STYLE_PROPERTIES) -> ::windows::core::Result<ID2D1InkStyle>;
    fn CreateGradientMesh(&mut self, patches: *const D2D1_GRADIENT_MESH_PATCH, patchescount: u32) -> ::windows::core::Result<ID2D1GradientMesh>;
    fn CreateImageSourceFromWic(&mut self, wicbitmapsource: &::core::option::Option<super::Imaging::IWICBitmapSource>, loadingoptions: D2D1_IMAGE_SOURCE_LOADING_OPTIONS, alphamode: Common::D2D1_ALPHA_MODE) -> ::windows::core::Result<ID2D1ImageSourceFromWic>;
    fn CreateLookupTable3D(&mut self, precision: D2D1_BUFFER_PRECISION, extents: *const u32, data: *const u8, datacount: u32, strides: *const u32) -> ::windows::core::Result<ID2D1LookupTable3D>;
    fn CreateImageSourceFromDxgi(&mut self, surfaces: *const ::core::option::Option<super::Dxgi::IDXGISurface>, surfacecount: u32, colorspace: super::Dxgi::Common::DXGI_COLOR_SPACE_TYPE, options: D2D1_IMAGE_SOURCE_FROM_DXGI_OPTIONS) -> ::windows::core::Result<ID2D1ImageSource>;
    fn GetGradientMeshWorldBounds(&mut self, gradientmesh: &::core::option::Option<ID2D1GradientMesh>) -> ::windows::core::Result<Common::D2D_RECT_F>;
    fn DrawInk(&mut self, ink: &::core::option::Option<ID2D1Ink>, brush: &::core::option::Option<ID2D1Brush>, inkstyle: &::core::option::Option<ID2D1InkStyle>);
    fn DrawGradientMesh(&mut self, gradientmesh: &::core::option::Option<ID2D1GradientMesh>);
    fn DrawGdiMetafile2(&mut self, gdimetafile: &::core::option::Option<ID2D1GdiMetafile>, destinationrectangle: *const Common::D2D_RECT_F, sourcerectangle: *const Common::D2D_RECT_F);
    fn CreateTransformedImageSource(&mut self, imagesource: &::core::option::Option<ID2D1ImageSource>, properties: *const D2D1_TRANSFORMED_IMAGE_SOURCE_PROPERTIES) -> ::windows::core::Result<ID2D1TransformedImageSource>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1DeviceContext2_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>() -> ID2D1DeviceContext2_Vtbl {
        unsafe extern "system" fn CreateInk<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, startpoint: *const D2D1_INK_POINT, ink: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateInk(::core::mem::transmute_copy(&startpoint)) {
                ::core::result::Result::Ok(ok__) => {
                    *ink = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateInkStyle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inkstyleproperties: *const D2D1_INK_STYLE_PROPERTIES, inkstyle: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateInkStyle(::core::mem::transmute_copy(&inkstyleproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *inkstyle = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateGradientMesh<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, patches: *const D2D1_GRADIENT_MESH_PATCH, patchescount: u32, gradientmesh: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateGradientMesh(::core::mem::transmute_copy(&patches), ::core::mem::transmute_copy(&patchescount)) {
                ::core::result::Result::Ok(ok__) => {
                    *gradientmesh = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateImageSourceFromWic<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, wicbitmapsource: ::windows::core::RawPtr, loadingoptions: D2D1_IMAGE_SOURCE_LOADING_OPTIONS, alphamode: Common::D2D1_ALPHA_MODE, imagesource: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateImageSourceFromWic(::core::mem::transmute(&wicbitmapsource), ::core::mem::transmute_copy(&loadingoptions), ::core::mem::transmute_copy(&alphamode)) {
                ::core::result::Result::Ok(ok__) => {
                    *imagesource = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateLookupTable3D<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, precision: D2D1_BUFFER_PRECISION, extents: *const u32, data: *const u8, datacount: u32, strides: *const u32, lookuptable: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateLookupTable3D(::core::mem::transmute_copy(&precision), ::core::mem::transmute_copy(&extents), ::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&datacount), ::core::mem::transmute_copy(&strides)) {
                ::core::result::Result::Ok(ok__) => {
                    *lookuptable = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateImageSourceFromDxgi<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, surfaces: *const ::windows::core::RawPtr, surfacecount: u32, colorspace: super::Dxgi::Common::DXGI_COLOR_SPACE_TYPE, options: D2D1_IMAGE_SOURCE_FROM_DXGI_OPTIONS, imagesource: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateImageSourceFromDxgi(::core::mem::transmute_copy(&surfaces), ::core::mem::transmute_copy(&surfacecount), ::core::mem::transmute_copy(&colorspace), ::core::mem::transmute_copy(&options)) {
                ::core::result::Result::Ok(ok__) => {
                    *imagesource = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn GetGradientMeshWorldBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gradientmesh: ::windows::core::RawPtr, pbounds: *mut Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetGradientMeshWorldBounds(::core::mem::transmute(&gradientmesh)) {
                ::core::result::Result::Ok(ok__) => {
                    *pbounds = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn DrawInk<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, ink: ::windows::core::RawPtr, brush: ::windows::core::RawPtr, inkstyle: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawInk(::core::mem::transmute(&ink), ::core::mem::transmute(&brush), ::core::mem::transmute(&inkstyle))
        }
        unsafe extern "system" fn DrawGradientMesh<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gradientmesh: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGradientMesh(::core::mem::transmute(&gradientmesh))
        }
        unsafe extern "system" fn DrawGdiMetafile2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gdimetafile: ::windows::core::RawPtr, destinationrectangle: *const Common::D2D_RECT_F, sourcerectangle: *const Common::D2D_RECT_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGdiMetafile2(::core::mem::transmute(&gdimetafile), ::core::mem::transmute_copy(&destinationrectangle), ::core::mem::transmute_copy(&sourcerectangle))
        }
        unsafe extern "system" fn CreateTransformedImageSource<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, imagesource: ::windows::core::RawPtr, properties: *const D2D1_TRANSFORMED_IMAGE_SOURCE_PROPERTIES, transformedimagesource: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateTransformedImageSource(::core::mem::transmute(&imagesource), ::core::mem::transmute_copy(&properties)) {
                ::core::result::Result::Ok(ok__) => {
                    *transformedimagesource = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1DeviceContext1_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateInk: CreateInk::<Identity, Impl, OFFSET>,
            CreateInkStyle: CreateInkStyle::<Identity, Impl, OFFSET>,
            CreateGradientMesh: CreateGradientMesh::<Identity, Impl, OFFSET>,
            CreateImageSourceFromWic: CreateImageSourceFromWic::<Identity, Impl, OFFSET>,
            CreateLookupTable3D: CreateLookupTable3D::<Identity, Impl, OFFSET>,
            CreateImageSourceFromDxgi: CreateImageSourceFromDxgi::<Identity, Impl, OFFSET>,
            GetGradientMeshWorldBounds: GetGradientMeshWorldBounds::<Identity, Impl, OFFSET>,
            DrawInk: DrawInk::<Identity, Impl, OFFSET>,
            DrawGradientMesh: DrawGradientMesh::<Identity, Impl, OFFSET>,
            DrawGdiMetafile2: DrawGdiMetafile2::<Identity, Impl, OFFSET>,
            CreateTransformedImageSource: CreateTransformedImageSource::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DeviceContext2 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext1 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1DeviceContext3_Impl: Sized + ID2D1Resource_Impl + ID2D1RenderTarget_Impl + ID2D1DeviceContext_Impl + ID2D1DeviceContext1_Impl + ID2D1DeviceContext2_Impl {
    fn CreateSpriteBatch(&mut self) -> ::windows::core::Result<ID2D1SpriteBatch>;
    fn DrawSpriteBatch(&mut self, spritebatch: &::core::option::Option<ID2D1SpriteBatch>, startindex: u32, spritecount: u32, bitmap: &::core::option::Option<ID2D1Bitmap>, interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE, spriteoptions: D2D1_SPRITE_OPTIONS);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1DeviceContext3_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext3_Impl, const OFFSET: isize>() -> ID2D1DeviceContext3_Vtbl {
        unsafe extern "system" fn CreateSpriteBatch<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext3_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, spritebatch: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateSpriteBatch() {
                ::core::result::Result::Ok(ok__) => {
                    *spritebatch = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn DrawSpriteBatch<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext3_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, spritebatch: ::windows::core::RawPtr, startindex: u32, spritecount: u32, bitmap: ::windows::core::RawPtr, interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE, spriteoptions: D2D1_SPRITE_OPTIONS) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawSpriteBatch(::core::mem::transmute(&spritebatch), ::core::mem::transmute_copy(&startindex), ::core::mem::transmute_copy(&spritecount), ::core::mem::transmute(&bitmap), ::core::mem::transmute_copy(&interpolationmode), ::core::mem::transmute_copy(&spriteoptions))
        }
        Self {
            base: ID2D1DeviceContext2_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateSpriteBatch: CreateSpriteBatch::<Identity, Impl, OFFSET>,
            DrawSpriteBatch: DrawSpriteBatch::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DeviceContext3 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext1 as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext2 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1DeviceContext4_Impl: Sized + ID2D1Resource_Impl + ID2D1RenderTarget_Impl + ID2D1DeviceContext_Impl + ID2D1DeviceContext1_Impl + ID2D1DeviceContext2_Impl + ID2D1DeviceContext3_Impl {
    fn CreateSvgGlyphStyle(&mut self) -> ::windows::core::Result<ID2D1SvgGlyphStyle>;
    fn DrawText2(&mut self, string: super::super::Foundation::PWSTR, stringlength: u32, textformat: &::core::option::Option<super::DirectWrite::IDWriteTextFormat>, layoutrect: *const Common::D2D_RECT_F, defaultfillbrush: &::core::option::Option<ID2D1Brush>, svgglyphstyle: &::core::option::Option<ID2D1SvgGlyphStyle>, colorpaletteindex: u32, options: D2D1_DRAW_TEXT_OPTIONS, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE);
    fn DrawTextLayout2(&mut self, origin: &Common::D2D_POINT_2F, textlayout: &::core::option::Option<super::DirectWrite::IDWriteTextLayout>, defaultfillbrush: &::core::option::Option<ID2D1Brush>, svgglyphstyle: &::core::option::Option<ID2D1SvgGlyphStyle>, colorpaletteindex: u32, options: D2D1_DRAW_TEXT_OPTIONS);
    fn DrawColorBitmapGlyphRun(&mut self, glyphimageformat: super::DirectWrite::DWRITE_GLYPH_IMAGE_FORMATS, baselineorigin: &Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE, bitmapsnapoption: D2D1_COLOR_BITMAP_GLYPH_SNAP_OPTION);
    fn DrawSvgGlyphRun(&mut self, baselineorigin: &Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, defaultfillbrush: &::core::option::Option<ID2D1Brush>, svgglyphstyle: &::core::option::Option<ID2D1SvgGlyphStyle>, colorpaletteindex: u32, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE);
    fn GetColorBitmapGlyphImage(&mut self, glyphimageformat: super::DirectWrite::DWRITE_GLYPH_IMAGE_FORMATS, glyphorigin: &Common::D2D_POINT_2F, fontface: &::core::option::Option<super::DirectWrite::IDWriteFontFace>, fontemsize: f32, glyphindex: u16, issideways: super::super::Foundation::BOOL, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, dpix: f32, dpiy: f32, glyphtransform: *mut super::super::super::Foundation::Numerics::Matrix3x2, glyphimage: *mut ::core::option::Option<ID2D1Image>) -> ::windows::core::Result<()>;
    fn GetSvgGlyphImage(&mut self, glyphorigin: &Common::D2D_POINT_2F, fontface: &::core::option::Option<super::DirectWrite::IDWriteFontFace>, fontemsize: f32, glyphindex: u16, issideways: super::super::Foundation::BOOL, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, defaultfillbrush: &::core::option::Option<ID2D1Brush>, svgglyphstyle: &::core::option::Option<ID2D1SvgGlyphStyle>, colorpaletteindex: u32, glyphtransform: *mut super::super::super::Foundation::Numerics::Matrix3x2, glyphimage: *mut ::core::option::Option<ID2D1CommandList>) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1DeviceContext4_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext4_Impl, const OFFSET: isize>() -> ID2D1DeviceContext4_Vtbl {
        unsafe extern "system" fn CreateSvgGlyphStyle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, svgglyphstyle: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateSvgGlyphStyle() {
                ::core::result::Result::Ok(ok__) => {
                    *svgglyphstyle = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn DrawText2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, string: super::super::Foundation::PWSTR, stringlength: u32, textformat: ::windows::core::RawPtr, layoutrect: *const Common::D2D_RECT_F, defaultfillbrush: ::windows::core::RawPtr, svgglyphstyle: ::windows::core::RawPtr, colorpaletteindex: u32, options: D2D1_DRAW_TEXT_OPTIONS, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawText2(::core::mem::transmute_copy(&string), ::core::mem::transmute_copy(&stringlength), ::core::mem::transmute(&textformat), ::core::mem::transmute_copy(&layoutrect), ::core::mem::transmute(&defaultfillbrush), ::core::mem::transmute(&svgglyphstyle), ::core::mem::transmute_copy(&colorpaletteindex), ::core::mem::transmute_copy(&options), ::core::mem::transmute_copy(&measuringmode))
        }
        unsafe extern "system" fn DrawTextLayout2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, origin: Common::D2D_POINT_2F, textlayout: ::windows::core::RawPtr, defaultfillbrush: ::windows::core::RawPtr, svgglyphstyle: ::windows::core::RawPtr, colorpaletteindex: u32, options: D2D1_DRAW_TEXT_OPTIONS) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawTextLayout2(::core::mem::transmute_copy(&origin), ::core::mem::transmute(&textlayout), ::core::mem::transmute(&defaultfillbrush), ::core::mem::transmute(&svgglyphstyle), ::core::mem::transmute_copy(&colorpaletteindex), ::core::mem::transmute_copy(&options))
        }
        unsafe extern "system" fn DrawColorBitmapGlyphRun<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, glyphimageformat: super::DirectWrite::DWRITE_GLYPH_IMAGE_FORMATS, baselineorigin: Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE, bitmapsnapoption: D2D1_COLOR_BITMAP_GLYPH_SNAP_OPTION) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawColorBitmapGlyphRun(::core::mem::transmute_copy(&glyphimageformat), ::core::mem::transmute_copy(&baselineorigin), ::core::mem::transmute_copy(&glyphrun), ::core::mem::transmute_copy(&measuringmode), ::core::mem::transmute_copy(&bitmapsnapoption))
        }
        unsafe extern "system" fn DrawSvgGlyphRun<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, baselineorigin: Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, defaultfillbrush: ::windows::core::RawPtr, svgglyphstyle: ::windows::core::RawPtr, colorpaletteindex: u32, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawSvgGlyphRun(::core::mem::transmute_copy(&baselineorigin), ::core::mem::transmute_copy(&glyphrun), ::core::mem::transmute(&defaultfillbrush), ::core::mem::transmute(&svgglyphstyle), ::core::mem::transmute_copy(&colorpaletteindex), ::core::mem::transmute_copy(&measuringmode))
        }
        unsafe extern "system" fn GetColorBitmapGlyphImage<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, glyphimageformat: super::DirectWrite::DWRITE_GLYPH_IMAGE_FORMATS, glyphorigin: Common::D2D_POINT_2F, fontface: ::windows::core::RawPtr, fontemsize: f32, glyphindex: u16, issideways: super::super::Foundation::BOOL, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, dpix: f32, dpiy: f32, glyphtransform: *mut super::super::super::Foundation::Numerics::Matrix3x2, glyphimage: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this)
                .GetColorBitmapGlyphImage(::core::mem::transmute_copy(&glyphimageformat), ::core::mem::transmute_copy(&glyphorigin), ::core::mem::transmute(&fontface), ::core::mem::transmute_copy(&fontemsize), ::core::mem::transmute_copy(&glyphindex), ::core::mem::transmute_copy(&issideways), ::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&dpix), ::core::mem::transmute_copy(&dpiy), ::core::mem::transmute_copy(&glyphtransform), ::core::mem::transmute_copy(&glyphimage))
                .into()
        }
        unsafe extern "system" fn GetSvgGlyphImage<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, glyphorigin: Common::D2D_POINT_2F, fontface: ::windows::core::RawPtr, fontemsize: f32, glyphindex: u16, issideways: super::super::Foundation::BOOL, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, defaultfillbrush: ::windows::core::RawPtr, svgglyphstyle: ::windows::core::RawPtr, colorpaletteindex: u32, glyphtransform: *mut super::super::super::Foundation::Numerics::Matrix3x2, glyphimage: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this)
                .GetSvgGlyphImage(::core::mem::transmute_copy(&glyphorigin), ::core::mem::transmute(&fontface), ::core::mem::transmute_copy(&fontemsize), ::core::mem::transmute_copy(&glyphindex), ::core::mem::transmute_copy(&issideways), ::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute(&defaultfillbrush), ::core::mem::transmute(&svgglyphstyle), ::core::mem::transmute_copy(&colorpaletteindex), ::core::mem::transmute_copy(&glyphtransform), ::core::mem::transmute_copy(&glyphimage))
                .into()
        }
        Self {
            base: ID2D1DeviceContext3_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateSvgGlyphStyle: CreateSvgGlyphStyle::<Identity, Impl, OFFSET>,
            DrawText2: DrawText2::<Identity, Impl, OFFSET>,
            DrawTextLayout2: DrawTextLayout2::<Identity, Impl, OFFSET>,
            DrawColorBitmapGlyphRun: DrawColorBitmapGlyphRun::<Identity, Impl, OFFSET>,
            DrawSvgGlyphRun: DrawSvgGlyphRun::<Identity, Impl, OFFSET>,
            GetColorBitmapGlyphImage: GetColorBitmapGlyphImage::<Identity, Impl, OFFSET>,
            GetSvgGlyphImage: GetSvgGlyphImage::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DeviceContext4 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext1 as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext2 as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext3 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
pub trait ID2D1DeviceContext5_Impl: Sized + ID2D1Resource_Impl + ID2D1RenderTarget_Impl + ID2D1DeviceContext_Impl + ID2D1DeviceContext1_Impl + ID2D1DeviceContext2_Impl + ID2D1DeviceContext3_Impl + ID2D1DeviceContext4_Impl {
    fn CreateSvgDocument(&mut self, inputxmlstream: &::core::option::Option<super::super::System::Com::IStream>, viewportsize: &Common::D2D_SIZE_F) -> ::windows::core::Result<ID2D1SvgDocument>;
    fn DrawSvgDocument(&mut self, svgdocument: &::core::option::Option<ID2D1SvgDocument>);
    fn CreateColorContextFromDxgiColorSpace(&mut self, colorspace: super::Dxgi::Common::DXGI_COLOR_SPACE_TYPE) -> ::windows::core::Result<ID2D1ColorContext1>;
    fn CreateColorContextFromSimpleColorProfile(&mut self, simpleprofile: *const D2D1_SIMPLE_COLOR_PROFILE) -> ::windows::core::Result<ID2D1ColorContext1>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
impl ID2D1DeviceContext5_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext5_Impl, const OFFSET: isize>() -> ID2D1DeviceContext5_Vtbl {
        unsafe extern "system" fn CreateSvgDocument<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext5_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inputxmlstream: ::windows::core::RawPtr, viewportsize: Common::D2D_SIZE_F, svgdocument: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateSvgDocument(::core::mem::transmute(&inputxmlstream), ::core::mem::transmute_copy(&viewportsize)) {
                ::core::result::Result::Ok(ok__) => {
                    *svgdocument = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn DrawSvgDocument<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext5_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, svgdocument: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawSvgDocument(::core::mem::transmute(&svgdocument))
        }
        unsafe extern "system" fn CreateColorContextFromDxgiColorSpace<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext5_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, colorspace: super::Dxgi::Common::DXGI_COLOR_SPACE_TYPE, colorcontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateColorContextFromDxgiColorSpace(::core::mem::transmute_copy(&colorspace)) {
                ::core::result::Result::Ok(ok__) => {
                    *colorcontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateColorContextFromSimpleColorProfile<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext5_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, simpleprofile: *const D2D1_SIMPLE_COLOR_PROFILE, colorcontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateColorContextFromSimpleColorProfile(::core::mem::transmute_copy(&simpleprofile)) {
                ::core::result::Result::Ok(ok__) => {
                    *colorcontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1DeviceContext4_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateSvgDocument: CreateSvgDocument::<Identity, Impl, OFFSET>,
            DrawSvgDocument: DrawSvgDocument::<Identity, Impl, OFFSET>,
            CreateColorContextFromDxgiColorSpace: CreateColorContextFromDxgiColorSpace::<Identity, Impl, OFFSET>,
            CreateColorContextFromSimpleColorProfile: CreateColorContextFromSimpleColorProfile::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DeviceContext5 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext1 as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext2 as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext3 as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext4 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
pub trait ID2D1DeviceContext6_Impl: Sized + ID2D1Resource_Impl + ID2D1RenderTarget_Impl + ID2D1DeviceContext_Impl + ID2D1DeviceContext1_Impl + ID2D1DeviceContext2_Impl + ID2D1DeviceContext3_Impl + ID2D1DeviceContext4_Impl + ID2D1DeviceContext5_Impl {
    fn BlendImage(&mut self, image: &::core::option::Option<ID2D1Image>, blendmode: Common::D2D1_BLEND_MODE, targetoffset: *const Common::D2D_POINT_2F, imagerectangle: *const Common::D2D_RECT_F, interpolationmode: D2D1_INTERPOLATION_MODE);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
impl ID2D1DeviceContext6_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext6_Impl, const OFFSET: isize>() -> ID2D1DeviceContext6_Vtbl {
        unsafe extern "system" fn BlendImage<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DeviceContext6_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: ::windows::core::RawPtr, blendmode: Common::D2D1_BLEND_MODE, targetoffset: *const Common::D2D_POINT_2F, imagerectangle: *const Common::D2D_RECT_F, interpolationmode: D2D1_INTERPOLATION_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).BlendImage(::core::mem::transmute(&image), ::core::mem::transmute_copy(&blendmode), ::core::mem::transmute_copy(&targetoffset), ::core::mem::transmute_copy(&imagerectangle), ::core::mem::transmute_copy(&interpolationmode))
        }
        Self { base: ID2D1DeviceContext5_Vtbl::new::<Identity, Impl, OFFSET>(), BlendImage: BlendImage::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DeviceContext6 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext1 as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext2 as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext3 as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext4 as ::windows::core::Interface>::IID || iid == &<ID2D1DeviceContext5 as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1DrawInfo_Impl: Sized + ID2D1RenderInfo_Impl {
    fn SetPixelShaderConstantBuffer(&mut self, buffer: *const u8, buffercount: u32) -> ::windows::core::Result<()>;
    fn SetResourceTexture(&mut self, textureindex: u32, resourcetexture: &::core::option::Option<ID2D1ResourceTexture>) -> ::windows::core::Result<()>;
    fn SetVertexShaderConstantBuffer(&mut self, buffer: *const u8, buffercount: u32) -> ::windows::core::Result<()>;
    fn SetPixelShader(&mut self, shaderid: *const ::windows::core::GUID, pixeloptions: D2D1_PIXEL_OPTIONS) -> ::windows::core::Result<()>;
    fn SetVertexProcessing(&mut self, vertexbuffer: &::core::option::Option<ID2D1VertexBuffer>, vertexoptions: D2D1_VERTEX_OPTIONS, blenddescription: *const D2D1_BLEND_DESCRIPTION, vertexrange: *const D2D1_VERTEX_RANGE, vertexshader: *const ::windows::core::GUID) -> ::windows::core::Result<()>;
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1DrawInfo_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawInfo_Impl, const OFFSET: isize>() -> ID2D1DrawInfo_Vtbl {
        unsafe extern "system" fn SetPixelShaderConstantBuffer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, buffer: *const u8, buffercount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetPixelShaderConstantBuffer(::core::mem::transmute_copy(&buffer), ::core::mem::transmute_copy(&buffercount)).into()
        }
        unsafe extern "system" fn SetResourceTexture<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, textureindex: u32, resourcetexture: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetResourceTexture(::core::mem::transmute_copy(&textureindex), ::core::mem::transmute(&resourcetexture)).into()
        }
        unsafe extern "system" fn SetVertexShaderConstantBuffer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, buffer: *const u8, buffercount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetVertexShaderConstantBuffer(::core::mem::transmute_copy(&buffer), ::core::mem::transmute_copy(&buffercount)).into()
        }
        unsafe extern "system" fn SetPixelShader<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, shaderid: *const ::windows::core::GUID, pixeloptions: D2D1_PIXEL_OPTIONS) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetPixelShader(::core::mem::transmute_copy(&shaderid), ::core::mem::transmute_copy(&pixeloptions)).into()
        }
        unsafe extern "system" fn SetVertexProcessing<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, vertexbuffer: ::windows::core::RawPtr, vertexoptions: D2D1_VERTEX_OPTIONS, blenddescription: *const D2D1_BLEND_DESCRIPTION, vertexrange: *const D2D1_VERTEX_RANGE, vertexshader: *const ::windows::core::GUID) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetVertexProcessing(::core::mem::transmute(&vertexbuffer), ::core::mem::transmute_copy(&vertexoptions), ::core::mem::transmute_copy(&blenddescription), ::core::mem::transmute_copy(&vertexrange), ::core::mem::transmute_copy(&vertexshader)).into()
        }
        Self {
            base: ID2D1RenderInfo_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetPixelShaderConstantBuffer: SetPixelShaderConstantBuffer::<Identity, Impl, OFFSET>,
            SetResourceTexture: SetResourceTexture::<Identity, Impl, OFFSET>,
            SetVertexShaderConstantBuffer: SetVertexShaderConstantBuffer::<Identity, Impl, OFFSET>,
            SetPixelShader: SetPixelShader::<Identity, Impl, OFFSET>,
            SetVertexProcessing: SetVertexProcessing::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DrawInfo as ::windows::core::Interface>::IID || iid == &<ID2D1RenderInfo as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1DrawTransform_Impl: Sized + ID2D1TransformNode_Impl + ID2D1Transform_Impl {
    fn SetDrawInfo(&mut self, drawinfo: &::core::option::Option<ID2D1DrawInfo>) -> ::windows::core::Result<()>;
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1DrawTransform_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawTransform_Impl, const OFFSET: isize>() -> ID2D1DrawTransform_Vtbl {
        unsafe extern "system" fn SetDrawInfo<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, drawinfo: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetDrawInfo(::core::mem::transmute(&drawinfo)).into()
        }
        Self { base: ID2D1Transform_Vtbl::new::<Identity, Impl, OFFSET>(), SetDrawInfo: SetDrawInfo::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DrawTransform as ::windows::core::Interface>::IID || iid == &<ID2D1TransformNode as ::windows::core::Interface>::IID || iid == &<ID2D1Transform as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_DirectWrite"))]
pub trait ID2D1DrawingStateBlock_Impl: Sized + ID2D1Resource_Impl {
    fn GetDescription(&mut self, statedescription: *mut D2D1_DRAWING_STATE_DESCRIPTION);
    fn SetDescription(&mut self, statedescription: *const D2D1_DRAWING_STATE_DESCRIPTION);
    fn SetTextRenderingParams(&mut self, textrenderingparams: &::core::option::Option<super::DirectWrite::IDWriteRenderingParams>);
    fn GetTextRenderingParams(&mut self, textrenderingparams: *mut ::core::option::Option<super::DirectWrite::IDWriteRenderingParams>);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_DirectWrite"))]
impl ID2D1DrawingStateBlock_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawingStateBlock_Impl, const OFFSET: isize>() -> ID2D1DrawingStateBlock_Vtbl {
        unsafe extern "system" fn GetDescription<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawingStateBlock_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, statedescription: *mut D2D1_DRAWING_STATE_DESCRIPTION) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDescription(::core::mem::transmute_copy(&statedescription))
        }
        unsafe extern "system" fn SetDescription<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawingStateBlock_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, statedescription: *const D2D1_DRAWING_STATE_DESCRIPTION) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetDescription(::core::mem::transmute_copy(&statedescription))
        }
        unsafe extern "system" fn SetTextRenderingParams<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawingStateBlock_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, textrenderingparams: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTextRenderingParams(::core::mem::transmute(&textrenderingparams))
        }
        unsafe extern "system" fn GetTextRenderingParams<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawingStateBlock_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, textrenderingparams: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTextRenderingParams(::core::mem::transmute_copy(&textrenderingparams))
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetDescription: GetDescription::<Identity, Impl, OFFSET>,
            SetDescription: SetDescription::<Identity, Impl, OFFSET>,
            SetTextRenderingParams: SetTextRenderingParams::<Identity, Impl, OFFSET>,
            GetTextRenderingParams: GetTextRenderingParams::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DrawingStateBlock as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_DirectWrite"))]
pub trait ID2D1DrawingStateBlock1_Impl: Sized + ID2D1Resource_Impl + ID2D1DrawingStateBlock_Impl {
    fn GetDescription2(&mut self, statedescription: *mut D2D1_DRAWING_STATE_DESCRIPTION1);
    fn SetDescription2(&mut self, statedescription: *const D2D1_DRAWING_STATE_DESCRIPTION1);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_DirectWrite"))]
impl ID2D1DrawingStateBlock1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawingStateBlock1_Impl, const OFFSET: isize>() -> ID2D1DrawingStateBlock1_Vtbl {
        unsafe extern "system" fn GetDescription2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawingStateBlock1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, statedescription: *mut D2D1_DRAWING_STATE_DESCRIPTION1) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDescription2(::core::mem::transmute_copy(&statedescription))
        }
        unsafe extern "system" fn SetDescription2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1DrawingStateBlock1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, statedescription: *const D2D1_DRAWING_STATE_DESCRIPTION1) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetDescription2(::core::mem::transmute_copy(&statedescription))
        }
        Self {
            base: ID2D1DrawingStateBlock_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetDescription2: GetDescription2::<Identity, Impl, OFFSET>,
            SetDescription2: SetDescription2::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1DrawingStateBlock1 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1DrawingStateBlock as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1Effect_Impl: Sized + ID2D1Properties_Impl {
    fn SetInput(&mut self, index: u32, input: &::core::option::Option<ID2D1Image>, invalidate: super::super::Foundation::BOOL);
    fn SetInputCount(&mut self, inputcount: u32) -> ::windows::core::Result<()>;
    fn GetInput(&mut self, index: u32, input: *mut ::core::option::Option<ID2D1Image>);
    fn GetInputCount(&mut self) -> u32;
    fn GetOutput(&mut self, outputimage: *mut ::core::option::Option<ID2D1Image>);
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1Effect_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Effect_Impl, const OFFSET: isize>() -> ID2D1Effect_Vtbl {
        unsafe extern "system" fn SetInput<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Effect_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32, input: ::windows::core::RawPtr, invalidate: super::super::Foundation::BOOL) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetInput(::core::mem::transmute_copy(&index), ::core::mem::transmute(&input), ::core::mem::transmute_copy(&invalidate))
        }
        unsafe extern "system" fn SetInputCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Effect_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inputcount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetInputCount(::core::mem::transmute_copy(&inputcount)).into()
        }
        unsafe extern "system" fn GetInput<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Effect_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32, input: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetInput(::core::mem::transmute_copy(&index), ::core::mem::transmute_copy(&input))
        }
        unsafe extern "system" fn GetInputCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Effect_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetInputCount()
        }
        unsafe extern "system" fn GetOutput<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Effect_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, outputimage: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetOutput(::core::mem::transmute_copy(&outputimage))
        }
        Self {
            base: ID2D1Properties_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetInput: SetInput::<Identity, Impl, OFFSET>,
            SetInputCount: SetInputCount::<Identity, Impl, OFFSET>,
            GetInput: GetInput::<Identity, Impl, OFFSET>,
            GetInputCount: GetInputCount::<Identity, Impl, OFFSET>,
            GetOutput: GetOutput::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Effect as ::windows::core::Interface>::IID || iid == &<ID2D1Properties as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1EffectContext_Impl: Sized {
    fn GetDpi(&mut self, dpix: *mut f32, dpiy: *mut f32);
    fn CreateEffect(&mut self, effectid: *const ::windows::core::GUID) -> ::windows::core::Result<ID2D1Effect>;
    fn GetMaximumSupportedFeatureLevel(&mut self, featurelevels: *const super::Direct3D::D3D_FEATURE_LEVEL, featurelevelscount: u32) -> ::windows::core::Result<super::Direct3D::D3D_FEATURE_LEVEL>;
    fn CreateTransformNodeFromEffect(&mut self, effect: &::core::option::Option<ID2D1Effect>) -> ::windows::core::Result<ID2D1TransformNode>;
    fn CreateBlendTransform(&mut self, numinputs: u32, blenddescription: *const D2D1_BLEND_DESCRIPTION) -> ::windows::core::Result<ID2D1BlendTransform>;
    fn CreateBorderTransform(&mut self, extendmodex: D2D1_EXTEND_MODE, extendmodey: D2D1_EXTEND_MODE) -> ::windows::core::Result<ID2D1BorderTransform>;
    fn CreateOffsetTransform(&mut self, offset: &super::super::Foundation::POINT) -> ::windows::core::Result<ID2D1OffsetTransform>;
    fn CreateBoundsAdjustmentTransform(&mut self, outputrectangle: *const super::super::Foundation::RECT) -> ::windows::core::Result<ID2D1BoundsAdjustmentTransform>;
    fn LoadPixelShader(&mut self, shaderid: *const ::windows::core::GUID, shaderbuffer: *const u8, shaderbuffercount: u32) -> ::windows::core::Result<()>;
    fn LoadVertexShader(&mut self, resourceid: *const ::windows::core::GUID, shaderbuffer: *const u8, shaderbuffercount: u32) -> ::windows::core::Result<()>;
    fn LoadComputeShader(&mut self, resourceid: *const ::windows::core::GUID, shaderbuffer: *const u8, shaderbuffercount: u32) -> ::windows::core::Result<()>;
    fn IsShaderLoaded(&mut self, shaderid: *const ::windows::core::GUID) -> super::super::Foundation::BOOL;
    fn CreateResourceTexture(&mut self, resourceid: *const ::windows::core::GUID, resourcetextureproperties: *const D2D1_RESOURCE_TEXTURE_PROPERTIES, data: *const u8, strides: *const u32, datasize: u32) -> ::windows::core::Result<ID2D1ResourceTexture>;
    fn FindResourceTexture(&mut self, resourceid: *const ::windows::core::GUID) -> ::windows::core::Result<ID2D1ResourceTexture>;
    fn CreateVertexBuffer(&mut self, vertexbufferproperties: *const D2D1_VERTEX_BUFFER_PROPERTIES, resourceid: *const ::windows::core::GUID, customvertexbufferproperties: *const D2D1_CUSTOM_VERTEX_BUFFER_PROPERTIES) -> ::windows::core::Result<ID2D1VertexBuffer>;
    fn FindVertexBuffer(&mut self, resourceid: *const ::windows::core::GUID) -> ::windows::core::Result<ID2D1VertexBuffer>;
    fn CreateColorContext(&mut self, space: D2D1_COLOR_SPACE, profile: *const u8, profilesize: u32) -> ::windows::core::Result<ID2D1ColorContext>;
    fn CreateColorContextFromFilename(&mut self, filename: super::super::Foundation::PWSTR) -> ::windows::core::Result<ID2D1ColorContext>;
    fn CreateColorContextFromWicColorContext(&mut self, wiccolorcontext: &::core::option::Option<super::Imaging::IWICColorContext>) -> ::windows::core::Result<ID2D1ColorContext>;
    fn CheckFeatureSupport(&mut self, feature: D2D1_FEATURE, featuresupportdata: *mut ::core::ffi::c_void, featuresupportdatasize: u32) -> ::windows::core::Result<()>;
    fn IsBufferPrecisionSupported(&mut self, bufferprecision: D2D1_BUFFER_PRECISION) -> super::super::Foundation::BOOL;
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1EffectContext_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>() -> ID2D1EffectContext_Vtbl {
        unsafe extern "system" fn GetDpi<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dpix: *mut f32, dpiy: *mut f32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDpi(::core::mem::transmute_copy(&dpix), ::core::mem::transmute_copy(&dpiy))
        }
        unsafe extern "system" fn CreateEffect<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, effectid: *const ::windows::core::GUID, effect: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateEffect(::core::mem::transmute_copy(&effectid)) {
                ::core::result::Result::Ok(ok__) => {
                    *effect = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn GetMaximumSupportedFeatureLevel<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, featurelevels: *const super::Direct3D::D3D_FEATURE_LEVEL, featurelevelscount: u32, maximumsupportedfeaturelevel: *mut super::Direct3D::D3D_FEATURE_LEVEL) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetMaximumSupportedFeatureLevel(::core::mem::transmute_copy(&featurelevels), ::core::mem::transmute_copy(&featurelevelscount)) {
                ::core::result::Result::Ok(ok__) => {
                    *maximumsupportedfeaturelevel = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateTransformNodeFromEffect<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, effect: ::windows::core::RawPtr, transformnode: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateTransformNodeFromEffect(::core::mem::transmute(&effect)) {
                ::core::result::Result::Ok(ok__) => {
                    *transformnode = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateBlendTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, numinputs: u32, blenddescription: *const D2D1_BLEND_DESCRIPTION, transform: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateBlendTransform(::core::mem::transmute_copy(&numinputs), ::core::mem::transmute_copy(&blenddescription)) {
                ::core::result::Result::Ok(ok__) => {
                    *transform = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateBorderTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, extendmodex: D2D1_EXTEND_MODE, extendmodey: D2D1_EXTEND_MODE, transform: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateBorderTransform(::core::mem::transmute_copy(&extendmodex), ::core::mem::transmute_copy(&extendmodey)) {
                ::core::result::Result::Ok(ok__) => {
                    *transform = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateOffsetTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, offset: super::super::Foundation::POINT, transform: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateOffsetTransform(::core::mem::transmute_copy(&offset)) {
                ::core::result::Result::Ok(ok__) => {
                    *transform = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateBoundsAdjustmentTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, outputrectangle: *const super::super::Foundation::RECT, transform: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateBoundsAdjustmentTransform(::core::mem::transmute_copy(&outputrectangle)) {
                ::core::result::Result::Ok(ok__) => {
                    *transform = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn LoadPixelShader<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, shaderid: *const ::windows::core::GUID, shaderbuffer: *const u8, shaderbuffercount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).LoadPixelShader(::core::mem::transmute_copy(&shaderid), ::core::mem::transmute_copy(&shaderbuffer), ::core::mem::transmute_copy(&shaderbuffercount)).into()
        }
        unsafe extern "system" fn LoadVertexShader<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, resourceid: *const ::windows::core::GUID, shaderbuffer: *const u8, shaderbuffercount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).LoadVertexShader(::core::mem::transmute_copy(&resourceid), ::core::mem::transmute_copy(&shaderbuffer), ::core::mem::transmute_copy(&shaderbuffercount)).into()
        }
        unsafe extern "system" fn LoadComputeShader<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, resourceid: *const ::windows::core::GUID, shaderbuffer: *const u8, shaderbuffercount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).LoadComputeShader(::core::mem::transmute_copy(&resourceid), ::core::mem::transmute_copy(&shaderbuffer), ::core::mem::transmute_copy(&shaderbuffercount)).into()
        }
        unsafe extern "system" fn IsShaderLoaded<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, shaderid: *const ::windows::core::GUID) -> super::super::Foundation::BOOL {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).IsShaderLoaded(::core::mem::transmute_copy(&shaderid))
        }
        unsafe extern "system" fn CreateResourceTexture<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, resourceid: *const ::windows::core::GUID, resourcetextureproperties: *const D2D1_RESOURCE_TEXTURE_PROPERTIES, data: *const u8, strides: *const u32, datasize: u32, resourcetexture: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateResourceTexture(::core::mem::transmute_copy(&resourceid), ::core::mem::transmute_copy(&resourcetextureproperties), ::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&strides), ::core::mem::transmute_copy(&datasize)) {
                ::core::result::Result::Ok(ok__) => {
                    *resourcetexture = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn FindResourceTexture<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, resourceid: *const ::windows::core::GUID, resourcetexture: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).FindResourceTexture(::core::mem::transmute_copy(&resourceid)) {
                ::core::result::Result::Ok(ok__) => {
                    *resourcetexture = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateVertexBuffer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, vertexbufferproperties: *const D2D1_VERTEX_BUFFER_PROPERTIES, resourceid: *const ::windows::core::GUID, customvertexbufferproperties: *const D2D1_CUSTOM_VERTEX_BUFFER_PROPERTIES, buffer: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateVertexBuffer(::core::mem::transmute_copy(&vertexbufferproperties), ::core::mem::transmute_copy(&resourceid), ::core::mem::transmute_copy(&customvertexbufferproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *buffer = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn FindVertexBuffer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, resourceid: *const ::windows::core::GUID, buffer: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).FindVertexBuffer(::core::mem::transmute_copy(&resourceid)) {
                ::core::result::Result::Ok(ok__) => {
                    *buffer = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateColorContext<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, space: D2D1_COLOR_SPACE, profile: *const u8, profilesize: u32, colorcontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateColorContext(::core::mem::transmute_copy(&space), ::core::mem::transmute_copy(&profile), ::core::mem::transmute_copy(&profilesize)) {
                ::core::result::Result::Ok(ok__) => {
                    *colorcontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateColorContextFromFilename<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, filename: super::super::Foundation::PWSTR, colorcontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateColorContextFromFilename(::core::mem::transmute_copy(&filename)) {
                ::core::result::Result::Ok(ok__) => {
                    *colorcontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateColorContextFromWicColorContext<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, wiccolorcontext: ::windows::core::RawPtr, colorcontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateColorContextFromWicColorContext(::core::mem::transmute(&wiccolorcontext)) {
                ::core::result::Result::Ok(ok__) => {
                    *colorcontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CheckFeatureSupport<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, feature: D2D1_FEATURE, featuresupportdata: *mut ::core::ffi::c_void, featuresupportdatasize: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).CheckFeatureSupport(::core::mem::transmute_copy(&feature), ::core::mem::transmute_copy(&featuresupportdata), ::core::mem::transmute_copy(&featuresupportdatasize)).into()
        }
        unsafe extern "system" fn IsBufferPrecisionSupported<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bufferprecision: D2D1_BUFFER_PRECISION) -> super::super::Foundation::BOOL {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).IsBufferPrecisionSupported(::core::mem::transmute_copy(&bufferprecision))
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            GetDpi: GetDpi::<Identity, Impl, OFFSET>,
            CreateEffect: CreateEffect::<Identity, Impl, OFFSET>,
            GetMaximumSupportedFeatureLevel: GetMaximumSupportedFeatureLevel::<Identity, Impl, OFFSET>,
            CreateTransformNodeFromEffect: CreateTransformNodeFromEffect::<Identity, Impl, OFFSET>,
            CreateBlendTransform: CreateBlendTransform::<Identity, Impl, OFFSET>,
            CreateBorderTransform: CreateBorderTransform::<Identity, Impl, OFFSET>,
            CreateOffsetTransform: CreateOffsetTransform::<Identity, Impl, OFFSET>,
            CreateBoundsAdjustmentTransform: CreateBoundsAdjustmentTransform::<Identity, Impl, OFFSET>,
            LoadPixelShader: LoadPixelShader::<Identity, Impl, OFFSET>,
            LoadVertexShader: LoadVertexShader::<Identity, Impl, OFFSET>,
            LoadComputeShader: LoadComputeShader::<Identity, Impl, OFFSET>,
            IsShaderLoaded: IsShaderLoaded::<Identity, Impl, OFFSET>,
            CreateResourceTexture: CreateResourceTexture::<Identity, Impl, OFFSET>,
            FindResourceTexture: FindResourceTexture::<Identity, Impl, OFFSET>,
            CreateVertexBuffer: CreateVertexBuffer::<Identity, Impl, OFFSET>,
            FindVertexBuffer: FindVertexBuffer::<Identity, Impl, OFFSET>,
            CreateColorContext: CreateColorContext::<Identity, Impl, OFFSET>,
            CreateColorContextFromFilename: CreateColorContextFromFilename::<Identity, Impl, OFFSET>,
            CreateColorContextFromWicColorContext: CreateColorContextFromWicColorContext::<Identity, Impl, OFFSET>,
            CheckFeatureSupport: CheckFeatureSupport::<Identity, Impl, OFFSET>,
            IsBufferPrecisionSupported: IsBufferPrecisionSupported::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1EffectContext as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1EffectContext1_Impl: Sized + ID2D1EffectContext_Impl {
    fn CreateLookupTable3D(&mut self, precision: D2D1_BUFFER_PRECISION, extents: *const u32, data: *const u8, datacount: u32, strides: *const u32) -> ::windows::core::Result<ID2D1LookupTable3D>;
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1EffectContext1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext1_Impl, const OFFSET: isize>() -> ID2D1EffectContext1_Vtbl {
        unsafe extern "system" fn CreateLookupTable3D<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, precision: D2D1_BUFFER_PRECISION, extents: *const u32, data: *const u8, datacount: u32, strides: *const u32, lookuptable: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateLookupTable3D(::core::mem::transmute_copy(&precision), ::core::mem::transmute_copy(&extents), ::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&datacount), ::core::mem::transmute_copy(&strides)) {
                ::core::result::Result::Ok(ok__) => {
                    *lookuptable = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1EffectContext_Vtbl::new::<Identity, Impl, OFFSET>(), CreateLookupTable3D: CreateLookupTable3D::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1EffectContext1 as ::windows::core::Interface>::IID || iid == &<ID2D1EffectContext as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1EffectContext2_Impl: Sized + ID2D1EffectContext_Impl + ID2D1EffectContext1_Impl {
    fn CreateColorContextFromDxgiColorSpace(&mut self, colorspace: super::Dxgi::Common::DXGI_COLOR_SPACE_TYPE) -> ::windows::core::Result<ID2D1ColorContext1>;
    fn CreateColorContextFromSimpleColorProfile(&mut self, simpleprofile: *const D2D1_SIMPLE_COLOR_PROFILE) -> ::windows::core::Result<ID2D1ColorContext1>;
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1EffectContext2_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext2_Impl, const OFFSET: isize>() -> ID2D1EffectContext2_Vtbl {
        unsafe extern "system" fn CreateColorContextFromDxgiColorSpace<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, colorspace: super::Dxgi::Common::DXGI_COLOR_SPACE_TYPE, colorcontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateColorContextFromDxgiColorSpace(::core::mem::transmute_copy(&colorspace)) {
                ::core::result::Result::Ok(ok__) => {
                    *colorcontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateColorContextFromSimpleColorProfile<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectContext2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, simpleprofile: *const D2D1_SIMPLE_COLOR_PROFILE, colorcontext: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateColorContextFromSimpleColorProfile(::core::mem::transmute_copy(&simpleprofile)) {
                ::core::result::Result::Ok(ok__) => {
                    *colorcontext = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1EffectContext1_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateColorContextFromDxgiColorSpace: CreateColorContextFromDxgiColorSpace::<Identity, Impl, OFFSET>,
            CreateColorContextFromSimpleColorProfile: CreateColorContextFromSimpleColorProfile::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1EffectContext2 as ::windows::core::Interface>::IID || iid == &<ID2D1EffectContext as ::windows::core::Interface>::IID || iid == &<ID2D1EffectContext1 as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1EffectImpl_Impl: Sized {
    fn Initialize(&mut self, effectcontext: &::core::option::Option<ID2D1EffectContext>, transformgraph: &::core::option::Option<ID2D1TransformGraph>) -> ::windows::core::Result<()>;
    fn PrepareForRender(&mut self, changetype: D2D1_CHANGE_TYPE) -> ::windows::core::Result<()>;
    fn SetGraph(&mut self, transformgraph: &::core::option::Option<ID2D1TransformGraph>) -> ::windows::core::Result<()>;
}
impl ID2D1EffectImpl_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectImpl_Impl, const OFFSET: isize>() -> ID2D1EffectImpl_Vtbl {
        unsafe extern "system" fn Initialize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectImpl_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, effectcontext: ::windows::core::RawPtr, transformgraph: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Initialize(::core::mem::transmute(&effectcontext), ::core::mem::transmute(&transformgraph)).into()
        }
        unsafe extern "system" fn PrepareForRender<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectImpl_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, changetype: D2D1_CHANGE_TYPE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).PrepareForRender(::core::mem::transmute_copy(&changetype)).into()
        }
        unsafe extern "system" fn SetGraph<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EffectImpl_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, transformgraph: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetGraph(::core::mem::transmute(&transformgraph)).into()
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            Initialize: Initialize::<Identity, Impl, OFFSET>,
            PrepareForRender: PrepareForRender::<Identity, Impl, OFFSET>,
            SetGraph: SetGraph::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1EffectImpl as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1EllipseGeometry_Impl: Sized + ID2D1Resource_Impl + ID2D1Geometry_Impl {
    fn GetEllipse(&mut self, ellipse: *mut D2D1_ELLIPSE);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1EllipseGeometry_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EllipseGeometry_Impl, const OFFSET: isize>() -> ID2D1EllipseGeometry_Vtbl {
        unsafe extern "system" fn GetEllipse<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1EllipseGeometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, ellipse: *mut D2D1_ELLIPSE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetEllipse(::core::mem::transmute_copy(&ellipse))
        }
        Self { base: ID2D1Geometry_Vtbl::new::<Identity, Impl, OFFSET>(), GetEllipse: GetEllipse::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1EllipseGeometry as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Geometry as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1Factory_Impl: Sized {
    fn ReloadSystemMetrics(&mut self) -> ::windows::core::Result<()>;
    fn GetDesktopDpi(&mut self, dpix: *mut f32, dpiy: *mut f32);
    fn CreateRectangleGeometry(&mut self, rectangle: *const Common::D2D_RECT_F) -> ::windows::core::Result<ID2D1RectangleGeometry>;
    fn CreateRoundedRectangleGeometry(&mut self, roundedrectangle: *const D2D1_ROUNDED_RECT) -> ::windows::core::Result<ID2D1RoundedRectangleGeometry>;
    fn CreateEllipseGeometry(&mut self, ellipse: *const D2D1_ELLIPSE) -> ::windows::core::Result<ID2D1EllipseGeometry>;
    fn CreateGeometryGroup(&mut self, fillmode: Common::D2D1_FILL_MODE, geometries: *const ::core::option::Option<ID2D1Geometry>, geometriescount: u32) -> ::windows::core::Result<ID2D1GeometryGroup>;
    fn CreateTransformedGeometry(&mut self, sourcegeometry: &::core::option::Option<ID2D1Geometry>, transform: *const super::super::super::Foundation::Numerics::Matrix3x2) -> ::windows::core::Result<ID2D1TransformedGeometry>;
    fn CreatePathGeometry(&mut self) -> ::windows::core::Result<ID2D1PathGeometry>;
    fn CreateStrokeStyle(&mut self, strokestyleproperties: *const D2D1_STROKE_STYLE_PROPERTIES, dashes: *const f32, dashescount: u32) -> ::windows::core::Result<ID2D1StrokeStyle>;
    fn CreateDrawingStateBlock(&mut self, drawingstatedescription: *const D2D1_DRAWING_STATE_DESCRIPTION, textrenderingparams: &::core::option::Option<super::DirectWrite::IDWriteRenderingParams>) -> ::windows::core::Result<ID2D1DrawingStateBlock>;
    fn CreateWicBitmapRenderTarget(&mut self, target: &::core::option::Option<super::Imaging::IWICBitmap>, rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES) -> ::windows::core::Result<ID2D1RenderTarget>;
    fn CreateHwndRenderTarget(&mut self, rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES, hwndrendertargetproperties: *const D2D1_HWND_RENDER_TARGET_PROPERTIES) -> ::windows::core::Result<ID2D1HwndRenderTarget>;
    fn CreateDxgiSurfaceRenderTarget(&mut self, dxgisurface: &::core::option::Option<super::Dxgi::IDXGISurface>, rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES) -> ::windows::core::Result<ID2D1RenderTarget>;
    fn CreateDCRenderTarget(&mut self, rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES) -> ::windows::core::Result<ID2D1DCRenderTarget>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1Factory_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>() -> ID2D1Factory_Vtbl {
        unsafe extern "system" fn ReloadSystemMetrics<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ReloadSystemMetrics().into()
        }
        unsafe extern "system" fn GetDesktopDpi<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dpix: *mut f32, dpiy: *mut f32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDesktopDpi(::core::mem::transmute_copy(&dpix), ::core::mem::transmute_copy(&dpiy))
        }
        unsafe extern "system" fn CreateRectangleGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rectangle: *const Common::D2D_RECT_F, rectanglegeometry: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateRectangleGeometry(::core::mem::transmute_copy(&rectangle)) {
                ::core::result::Result::Ok(ok__) => {
                    *rectanglegeometry = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateRoundedRectangleGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, roundedrectangle: *const D2D1_ROUNDED_RECT, roundedrectanglegeometry: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateRoundedRectangleGeometry(::core::mem::transmute_copy(&roundedrectangle)) {
                ::core::result::Result::Ok(ok__) => {
                    *roundedrectanglegeometry = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateEllipseGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, ellipse: *const D2D1_ELLIPSE, ellipsegeometry: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateEllipseGeometry(::core::mem::transmute_copy(&ellipse)) {
                ::core::result::Result::Ok(ok__) => {
                    *ellipsegeometry = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateGeometryGroup<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, fillmode: Common::D2D1_FILL_MODE, geometries: *const ::windows::core::RawPtr, geometriescount: u32, geometrygroup: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateGeometryGroup(::core::mem::transmute_copy(&fillmode), ::core::mem::transmute_copy(&geometries), ::core::mem::transmute_copy(&geometriescount)) {
                ::core::result::Result::Ok(ok__) => {
                    *geometrygroup = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateTransformedGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, sourcegeometry: ::windows::core::RawPtr, transform: *const super::super::super::Foundation::Numerics::Matrix3x2, transformedgeometry: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateTransformedGeometry(::core::mem::transmute(&sourcegeometry), ::core::mem::transmute_copy(&transform)) {
                ::core::result::Result::Ok(ok__) => {
                    *transformedgeometry = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreatePathGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, pathgeometry: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreatePathGeometry() {
                ::core::result::Result::Ok(ok__) => {
                    *pathgeometry = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateStrokeStyle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, strokestyleproperties: *const D2D1_STROKE_STYLE_PROPERTIES, dashes: *const f32, dashescount: u32, strokestyle: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateStrokeStyle(::core::mem::transmute_copy(&strokestyleproperties), ::core::mem::transmute_copy(&dashes), ::core::mem::transmute_copy(&dashescount)) {
                ::core::result::Result::Ok(ok__) => {
                    *strokestyle = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateDrawingStateBlock<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, drawingstatedescription: *const D2D1_DRAWING_STATE_DESCRIPTION, textrenderingparams: ::windows::core::RawPtr, drawingstateblock: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDrawingStateBlock(::core::mem::transmute_copy(&drawingstatedescription), ::core::mem::transmute(&textrenderingparams)) {
                ::core::result::Result::Ok(ok__) => {
                    *drawingstateblock = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateWicBitmapRenderTarget<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, target: ::windows::core::RawPtr, rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES, rendertarget: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateWicBitmapRenderTarget(::core::mem::transmute(&target), ::core::mem::transmute_copy(&rendertargetproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *rendertarget = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateHwndRenderTarget<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES, hwndrendertargetproperties: *const D2D1_HWND_RENDER_TARGET_PROPERTIES, hwndrendertarget: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateHwndRenderTarget(::core::mem::transmute_copy(&rendertargetproperties), ::core::mem::transmute_copy(&hwndrendertargetproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *hwndrendertarget = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateDxgiSurfaceRenderTarget<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dxgisurface: ::windows::core::RawPtr, rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES, rendertarget: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDxgiSurfaceRenderTarget(::core::mem::transmute(&dxgisurface), ::core::mem::transmute_copy(&rendertargetproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *rendertarget = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateDCRenderTarget<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES, dcrendertarget: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDCRenderTarget(::core::mem::transmute_copy(&rendertargetproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *dcrendertarget = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            ReloadSystemMetrics: ReloadSystemMetrics::<Identity, Impl, OFFSET>,
            GetDesktopDpi: GetDesktopDpi::<Identity, Impl, OFFSET>,
            CreateRectangleGeometry: CreateRectangleGeometry::<Identity, Impl, OFFSET>,
            CreateRoundedRectangleGeometry: CreateRoundedRectangleGeometry::<Identity, Impl, OFFSET>,
            CreateEllipseGeometry: CreateEllipseGeometry::<Identity, Impl, OFFSET>,
            CreateGeometryGroup: CreateGeometryGroup::<Identity, Impl, OFFSET>,
            CreateTransformedGeometry: CreateTransformedGeometry::<Identity, Impl, OFFSET>,
            CreatePathGeometry: CreatePathGeometry::<Identity, Impl, OFFSET>,
            CreateStrokeStyle: CreateStrokeStyle::<Identity, Impl, OFFSET>,
            CreateDrawingStateBlock: CreateDrawingStateBlock::<Identity, Impl, OFFSET>,
            CreateWicBitmapRenderTarget: CreateWicBitmapRenderTarget::<Identity, Impl, OFFSET>,
            CreateHwndRenderTarget: CreateHwndRenderTarget::<Identity, Impl, OFFSET>,
            CreateDxgiSurfaceRenderTarget: CreateDxgiSurfaceRenderTarget::<Identity, Impl, OFFSET>,
            CreateDCRenderTarget: CreateDCRenderTarget::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Factory as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
pub trait ID2D1Factory1_Impl: Sized + ID2D1Factory_Impl {
    fn CreateDevice(&mut self, dxgidevice: &::core::option::Option<super::Dxgi::IDXGIDevice>) -> ::windows::core::Result<ID2D1Device>;
    fn CreateStrokeStyle2(&mut self, strokestyleproperties: *const D2D1_STROKE_STYLE_PROPERTIES1, dashes: *const f32, dashescount: u32) -> ::windows::core::Result<ID2D1StrokeStyle1>;
    fn CreatePathGeometry2(&mut self) -> ::windows::core::Result<ID2D1PathGeometry1>;
    fn CreateDrawingStateBlock2(&mut self, drawingstatedescription: *const D2D1_DRAWING_STATE_DESCRIPTION1, textrenderingparams: &::core::option::Option<super::DirectWrite::IDWriteRenderingParams>) -> ::windows::core::Result<ID2D1DrawingStateBlock1>;
    fn CreateGdiMetafile(&mut self, metafilestream: &::core::option::Option<super::super::System::Com::IStream>) -> ::windows::core::Result<ID2D1GdiMetafile>;
    fn RegisterEffectFromStream(&mut self, classid: *const ::windows::core::GUID, propertyxml: &::core::option::Option<super::super::System::Com::IStream>, bindings: *const D2D1_PROPERTY_BINDING, bindingscount: u32, effectfactory: &PD2D1_EFFECT_FACTORY) -> ::windows::core::Result<()>;
    fn RegisterEffectFromString(&mut self, classid: *const ::windows::core::GUID, propertyxml: super::super::Foundation::PWSTR, bindings: *const D2D1_PROPERTY_BINDING, bindingscount: u32, effectfactory: &PD2D1_EFFECT_FACTORY) -> ::windows::core::Result<()>;
    fn UnregisterEffect(&mut self, classid: *const ::windows::core::GUID) -> ::windows::core::Result<()>;
    fn GetRegisteredEffects(&mut self, effects: *mut ::windows::core::GUID, effectscount: u32, effectsreturned: *mut u32, effectsregistered: *mut u32) -> ::windows::core::Result<()>;
    fn GetEffectProperties(&mut self, effectid: *const ::windows::core::GUID) -> ::windows::core::Result<ID2D1Properties>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
impl ID2D1Factory1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>() -> ID2D1Factory1_Vtbl {
        unsafe extern "system" fn CreateDevice<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dxgidevice: ::windows::core::RawPtr, d2ddevice: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDevice(::core::mem::transmute(&dxgidevice)) {
                ::core::result::Result::Ok(ok__) => {
                    *d2ddevice = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateStrokeStyle2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, strokestyleproperties: *const D2D1_STROKE_STYLE_PROPERTIES1, dashes: *const f32, dashescount: u32, strokestyle: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateStrokeStyle2(::core::mem::transmute_copy(&strokestyleproperties), ::core::mem::transmute_copy(&dashes), ::core::mem::transmute_copy(&dashescount)) {
                ::core::result::Result::Ok(ok__) => {
                    *strokestyle = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreatePathGeometry2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, pathgeometry: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreatePathGeometry2() {
                ::core::result::Result::Ok(ok__) => {
                    *pathgeometry = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateDrawingStateBlock2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, drawingstatedescription: *const D2D1_DRAWING_STATE_DESCRIPTION1, textrenderingparams: ::windows::core::RawPtr, drawingstateblock: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDrawingStateBlock2(::core::mem::transmute_copy(&drawingstatedescription), ::core::mem::transmute(&textrenderingparams)) {
                ::core::result::Result::Ok(ok__) => {
                    *drawingstateblock = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateGdiMetafile<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, metafilestream: ::windows::core::RawPtr, metafile: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateGdiMetafile(::core::mem::transmute(&metafilestream)) {
                ::core::result::Result::Ok(ok__) => {
                    *metafile = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn RegisterEffectFromStream<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, classid: *const ::windows::core::GUID, propertyxml: ::windows::core::RawPtr, bindings: *const D2D1_PROPERTY_BINDING, bindingscount: u32, effectfactory: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RegisterEffectFromStream(::core::mem::transmute_copy(&classid), ::core::mem::transmute(&propertyxml), ::core::mem::transmute_copy(&bindings), ::core::mem::transmute_copy(&bindingscount), ::core::mem::transmute_copy(&effectfactory)).into()
        }
        unsafe extern "system" fn RegisterEffectFromString<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, classid: *const ::windows::core::GUID, propertyxml: super::super::Foundation::PWSTR, bindings: *const D2D1_PROPERTY_BINDING, bindingscount: u32, effectfactory: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RegisterEffectFromString(::core::mem::transmute_copy(&classid), ::core::mem::transmute_copy(&propertyxml), ::core::mem::transmute_copy(&bindings), ::core::mem::transmute_copy(&bindingscount), ::core::mem::transmute_copy(&effectfactory)).into()
        }
        unsafe extern "system" fn UnregisterEffect<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, classid: *const ::windows::core::GUID) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).UnregisterEffect(::core::mem::transmute_copy(&classid)).into()
        }
        unsafe extern "system" fn GetRegisteredEffects<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, effects: *mut ::windows::core::GUID, effectscount: u32, effectsreturned: *mut u32, effectsregistered: *mut u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetRegisteredEffects(::core::mem::transmute_copy(&effects), ::core::mem::transmute_copy(&effectscount), ::core::mem::transmute_copy(&effectsreturned), ::core::mem::transmute_copy(&effectsregistered)).into()
        }
        unsafe extern "system" fn GetEffectProperties<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, effectid: *const ::windows::core::GUID, properties: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetEffectProperties(::core::mem::transmute_copy(&effectid)) {
                ::core::result::Result::Ok(ok__) => {
                    *properties = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1Factory_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateDevice: CreateDevice::<Identity, Impl, OFFSET>,
            CreateStrokeStyle2: CreateStrokeStyle2::<Identity, Impl, OFFSET>,
            CreatePathGeometry2: CreatePathGeometry2::<Identity, Impl, OFFSET>,
            CreateDrawingStateBlock2: CreateDrawingStateBlock2::<Identity, Impl, OFFSET>,
            CreateGdiMetafile: CreateGdiMetafile::<Identity, Impl, OFFSET>,
            RegisterEffectFromStream: RegisterEffectFromStream::<Identity, Impl, OFFSET>,
            RegisterEffectFromString: RegisterEffectFromString::<Identity, Impl, OFFSET>,
            UnregisterEffect: UnregisterEffect::<Identity, Impl, OFFSET>,
            GetRegisteredEffects: GetRegisteredEffects::<Identity, Impl, OFFSET>,
            GetEffectProperties: GetEffectProperties::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Factory1 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
pub trait ID2D1Factory2_Impl: Sized + ID2D1Factory_Impl + ID2D1Factory1_Impl {
    fn CreateDevice2(&mut self, dxgidevice: &::core::option::Option<super::Dxgi::IDXGIDevice>) -> ::windows::core::Result<ID2D1Device1>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
impl ID2D1Factory2_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory2_Impl, const OFFSET: isize>() -> ID2D1Factory2_Vtbl {
        unsafe extern "system" fn CreateDevice2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory2_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dxgidevice: ::windows::core::RawPtr, d2ddevice1: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDevice2(::core::mem::transmute(&dxgidevice)) {
                ::core::result::Result::Ok(ok__) => {
                    *d2ddevice1 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1Factory1_Vtbl::new::<Identity, Impl, OFFSET>(), CreateDevice2: CreateDevice2::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Factory2 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory as ::windows::core::Interface>::IID || iid == &<ID2D1Factory1 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
pub trait ID2D1Factory3_Impl: Sized + ID2D1Factory_Impl + ID2D1Factory1_Impl + ID2D1Factory2_Impl {
    fn CreateDevice3(&mut self, dxgidevice: &::core::option::Option<super::Dxgi::IDXGIDevice>) -> ::windows::core::Result<ID2D1Device2>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
impl ID2D1Factory3_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory3_Impl, const OFFSET: isize>() -> ID2D1Factory3_Vtbl {
        unsafe extern "system" fn CreateDevice3<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory3_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dxgidevice: ::windows::core::RawPtr, d2ddevice2: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDevice3(::core::mem::transmute(&dxgidevice)) {
                ::core::result::Result::Ok(ok__) => {
                    *d2ddevice2 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1Factory2_Vtbl::new::<Identity, Impl, OFFSET>(), CreateDevice3: CreateDevice3::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Factory3 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory as ::windows::core::Interface>::IID || iid == &<ID2D1Factory1 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory2 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
pub trait ID2D1Factory4_Impl: Sized + ID2D1Factory_Impl + ID2D1Factory1_Impl + ID2D1Factory2_Impl + ID2D1Factory3_Impl {
    fn CreateDevice4(&mut self, dxgidevice: &::core::option::Option<super::Dxgi::IDXGIDevice>) -> ::windows::core::Result<ID2D1Device3>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
impl ID2D1Factory4_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory4_Impl, const OFFSET: isize>() -> ID2D1Factory4_Vtbl {
        unsafe extern "system" fn CreateDevice4<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory4_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dxgidevice: ::windows::core::RawPtr, d2ddevice3: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDevice4(::core::mem::transmute(&dxgidevice)) {
                ::core::result::Result::Ok(ok__) => {
                    *d2ddevice3 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1Factory3_Vtbl::new::<Identity, Impl, OFFSET>(), CreateDevice4: CreateDevice4::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Factory4 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory as ::windows::core::Interface>::IID || iid == &<ID2D1Factory1 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory2 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory3 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
pub trait ID2D1Factory5_Impl: Sized + ID2D1Factory_Impl + ID2D1Factory1_Impl + ID2D1Factory2_Impl + ID2D1Factory3_Impl + ID2D1Factory4_Impl {
    fn CreateDevice5(&mut self, dxgidevice: &::core::option::Option<super::Dxgi::IDXGIDevice>) -> ::windows::core::Result<ID2D1Device4>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
impl ID2D1Factory5_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory5_Impl, const OFFSET: isize>() -> ID2D1Factory5_Vtbl {
        unsafe extern "system" fn CreateDevice5<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory5_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dxgidevice: ::windows::core::RawPtr, d2ddevice4: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDevice5(::core::mem::transmute(&dxgidevice)) {
                ::core::result::Result::Ok(ok__) => {
                    *d2ddevice4 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1Factory4_Vtbl::new::<Identity, Impl, OFFSET>(), CreateDevice5: CreateDevice5::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Factory5 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory as ::windows::core::Interface>::IID || iid == &<ID2D1Factory1 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory2 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory3 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory4 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
pub trait ID2D1Factory6_Impl: Sized + ID2D1Factory_Impl + ID2D1Factory1_Impl + ID2D1Factory2_Impl + ID2D1Factory3_Impl + ID2D1Factory4_Impl + ID2D1Factory5_Impl {
    fn CreateDevice6(&mut self, dxgidevice: &::core::option::Option<super::Dxgi::IDXGIDevice>) -> ::windows::core::Result<ID2D1Device5>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
impl ID2D1Factory6_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory6_Impl, const OFFSET: isize>() -> ID2D1Factory6_Vtbl {
        unsafe extern "system" fn CreateDevice6<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory6_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dxgidevice: ::windows::core::RawPtr, d2ddevice5: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDevice6(::core::mem::transmute(&dxgidevice)) {
                ::core::result::Result::Ok(ok__) => {
                    *d2ddevice5 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1Factory5_Vtbl::new::<Identity, Impl, OFFSET>(), CreateDevice6: CreateDevice6::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Factory6 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory as ::windows::core::Interface>::IID || iid == &<ID2D1Factory1 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory2 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory3 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory4 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory5 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
pub trait ID2D1Factory7_Impl: Sized + ID2D1Factory_Impl + ID2D1Factory1_Impl + ID2D1Factory2_Impl + ID2D1Factory3_Impl + ID2D1Factory4_Impl + ID2D1Factory5_Impl + ID2D1Factory6_Impl {
    fn CreateDevice7(&mut self, dxgidevice: &::core::option::Option<super::Dxgi::IDXGIDevice>) -> ::windows::core::Result<ID2D1Device6>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging", feature = "Win32_System_Com"))]
impl ID2D1Factory7_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory7_Impl, const OFFSET: isize>() -> ID2D1Factory7_Vtbl {
        unsafe extern "system" fn CreateDevice7<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Factory7_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dxgidevice: ::windows::core::RawPtr, d2ddevice6: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateDevice7(::core::mem::transmute(&dxgidevice)) {
                ::core::result::Result::Ok(ok__) => {
                    *d2ddevice6 = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1Factory6_Vtbl::new::<Identity, Impl, OFFSET>(), CreateDevice7: CreateDevice7::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Factory7 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory as ::windows::core::Interface>::IID || iid == &<ID2D1Factory1 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory2 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory3 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory4 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory5 as ::windows::core::Interface>::IID || iid == &<ID2D1Factory6 as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub trait ID2D1GdiInteropRenderTarget_Impl: Sized {
    fn GetDC(&mut self, mode: D2D1_DC_INITIALIZE_MODE) -> ::windows::core::Result<super::Gdi::HDC>;
    fn ReleaseDC(&mut self, update: *const super::super::Foundation::RECT) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ID2D1GdiInteropRenderTarget_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiInteropRenderTarget_Impl, const OFFSET: isize>() -> ID2D1GdiInteropRenderTarget_Vtbl {
        unsafe extern "system" fn GetDC<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiInteropRenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, mode: D2D1_DC_INITIALIZE_MODE, hdc: *mut super::Gdi::HDC) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetDC(::core::mem::transmute_copy(&mode)) {
                ::core::result::Result::Ok(ok__) => {
                    *hdc = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn ReleaseDC<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiInteropRenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, update: *const super::super::Foundation::RECT) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ReleaseDC(::core::mem::transmute_copy(&update)).into()
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            GetDC: GetDC::<Identity, Impl, OFFSET>,
            ReleaseDC: ReleaseDC::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GdiInteropRenderTarget as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
pub trait ID2D1GdiMetafile_Impl: Sized + ID2D1Resource_Impl {
    fn Stream(&mut self, sink: &::core::option::Option<ID2D1GdiMetafileSink>) -> ::windows::core::Result<()>;
    fn GetBounds(&mut self) -> ::windows::core::Result<Common::D2D_RECT_F>;
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
impl ID2D1GdiMetafile_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiMetafile_Impl, const OFFSET: isize>() -> ID2D1GdiMetafile_Vtbl {
        unsafe extern "system" fn Stream<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiMetafile_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, sink: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Stream(::core::mem::transmute(&sink)).into()
        }
        unsafe extern "system" fn GetBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiMetafile_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bounds: *mut Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetBounds() {
                ::core::result::Result::Ok(ok__) => {
                    *bounds = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            Stream: Stream::<Identity, Impl, OFFSET>,
            GetBounds: GetBounds::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GdiMetafile as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
pub trait ID2D1GdiMetafile1_Impl: Sized + ID2D1Resource_Impl + ID2D1GdiMetafile_Impl {
    fn GetDpi(&mut self, dpix: *mut f32, dpiy: *mut f32) -> ::windows::core::Result<()>;
    fn GetSourceBounds(&mut self) -> ::windows::core::Result<Common::D2D_RECT_F>;
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
impl ID2D1GdiMetafile1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiMetafile1_Impl, const OFFSET: isize>() -> ID2D1GdiMetafile1_Vtbl {
        unsafe extern "system" fn GetDpi<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiMetafile1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dpix: *mut f32, dpiy: *mut f32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDpi(::core::mem::transmute_copy(&dpix), ::core::mem::transmute_copy(&dpiy)).into()
        }
        unsafe extern "system" fn GetSourceBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiMetafile1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bounds: *mut Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetSourceBounds() {
                ::core::result::Result::Ok(ok__) => {
                    *bounds = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1GdiMetafile_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetDpi: GetDpi::<Identity, Impl, OFFSET>,
            GetSourceBounds: GetSourceBounds::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GdiMetafile1 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1GdiMetafile as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1GdiMetafileSink_Impl: Sized {
    fn ProcessRecord(&mut self, recordtype: u32, recorddata: *const ::core::ffi::c_void, recorddatasize: u32) -> ::windows::core::Result<()>;
}
impl ID2D1GdiMetafileSink_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiMetafileSink_Impl, const OFFSET: isize>() -> ID2D1GdiMetafileSink_Vtbl {
        unsafe extern "system" fn ProcessRecord<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiMetafileSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, recordtype: u32, recorddata: *const ::core::ffi::c_void, recorddatasize: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ProcessRecord(::core::mem::transmute_copy(&recordtype), ::core::mem::transmute_copy(&recorddata), ::core::mem::transmute_copy(&recorddatasize)).into()
        }
        Self { base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(), ProcessRecord: ProcessRecord::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GdiMetafileSink as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1GdiMetafileSink1_Impl: Sized + ID2D1GdiMetafileSink_Impl {
    fn ProcessRecord2(&mut self, recordtype: u32, recorddata: *const ::core::ffi::c_void, recorddatasize: u32, flags: u32) -> ::windows::core::Result<()>;
}
impl ID2D1GdiMetafileSink1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiMetafileSink1_Impl, const OFFSET: isize>() -> ID2D1GdiMetafileSink1_Vtbl {
        unsafe extern "system" fn ProcessRecord2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GdiMetafileSink1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, recordtype: u32, recorddata: *const ::core::ffi::c_void, recorddatasize: u32, flags: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ProcessRecord2(::core::mem::transmute_copy(&recordtype), ::core::mem::transmute_copy(&recorddata), ::core::mem::transmute_copy(&recorddatasize), ::core::mem::transmute_copy(&flags)).into()
        }
        Self { base: ID2D1GdiMetafileSink_Vtbl::new::<Identity, Impl, OFFSET>(), ProcessRecord2: ProcessRecord2::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GdiMetafileSink1 as ::windows::core::Interface>::IID || iid == &<ID2D1GdiMetafileSink as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1Geometry_Impl: Sized + ID2D1Resource_Impl {
    fn GetBounds(&mut self, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2) -> ::windows::core::Result<Common::D2D_RECT_F>;
    fn GetWidenedBounds(&mut self, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32) -> ::windows::core::Result<Common::D2D_RECT_F>;
    fn StrokeContainsPoint(&mut self, point: &Common::D2D_POINT_2F, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32) -> ::windows::core::Result<super::super::Foundation::BOOL>;
    fn FillContainsPoint(&mut self, point: &Common::D2D_POINT_2F, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32) -> ::windows::core::Result<super::super::Foundation::BOOL>;
    fn CompareWithGeometry(&mut self, inputgeometry: &::core::option::Option<ID2D1Geometry>, inputgeometrytransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32) -> ::windows::core::Result<D2D1_GEOMETRY_RELATION>;
    fn Simplify(&mut self, simplificationoption: D2D1_GEOMETRY_SIMPLIFICATION_OPTION, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, geometrysink: &::core::option::Option<Common::ID2D1SimplifiedGeometrySink>) -> ::windows::core::Result<()>;
    fn Tessellate(&mut self, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, tessellationsink: &::core::option::Option<ID2D1TessellationSink>) -> ::windows::core::Result<()>;
    fn CombineWithGeometry(&mut self, inputgeometry: &::core::option::Option<ID2D1Geometry>, combinemode: D2D1_COMBINE_MODE, inputgeometrytransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, geometrysink: &::core::option::Option<Common::ID2D1SimplifiedGeometrySink>) -> ::windows::core::Result<()>;
    fn Outline(&mut self, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, geometrysink: &::core::option::Option<Common::ID2D1SimplifiedGeometrySink>) -> ::windows::core::Result<()>;
    fn ComputeArea(&mut self, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32) -> ::windows::core::Result<f32>;
    fn ComputeLength(&mut self, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32) -> ::windows::core::Result<f32>;
    fn ComputePointAtLength(&mut self, length: f32, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, point: *mut Common::D2D_POINT_2F, unittangentvector: *mut Common::D2D_POINT_2F) -> ::windows::core::Result<()>;
    fn Widen(&mut self, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, geometrysink: &::core::option::Option<Common::ID2D1SimplifiedGeometrySink>) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1Geometry_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>() -> ID2D1Geometry_Vtbl {
        unsafe extern "system" fn GetBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, bounds: *mut Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetBounds(::core::mem::transmute_copy(&worldtransform)) {
                ::core::result::Result::Ok(ok__) => {
                    *bounds = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn GetWidenedBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, strokewidth: f32, strokestyle: ::windows::core::RawPtr, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, bounds: *mut Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetWidenedBounds(::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle), ::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance)) {
                ::core::result::Result::Ok(ok__) => {
                    *bounds = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn StrokeContainsPoint<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, point: Common::D2D_POINT_2F, strokewidth: f32, strokestyle: ::windows::core::RawPtr, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, contains: *mut super::super::Foundation::BOOL) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).StrokeContainsPoint(::core::mem::transmute_copy(&point), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle), ::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance)) {
                ::core::result::Result::Ok(ok__) => {
                    *contains = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn FillContainsPoint<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, point: Common::D2D_POINT_2F, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, contains: *mut super::super::Foundation::BOOL) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).FillContainsPoint(::core::mem::transmute_copy(&point), ::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance)) {
                ::core::result::Result::Ok(ok__) => {
                    *contains = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CompareWithGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inputgeometry: ::windows::core::RawPtr, inputgeometrytransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, relation: *mut D2D1_GEOMETRY_RELATION) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CompareWithGeometry(::core::mem::transmute(&inputgeometry), ::core::mem::transmute_copy(&inputgeometrytransform), ::core::mem::transmute_copy(&flatteningtolerance)) {
                ::core::result::Result::Ok(ok__) => {
                    *relation = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn Simplify<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, simplificationoption: D2D1_GEOMETRY_SIMPLIFICATION_OPTION, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, geometrysink: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Simplify(::core::mem::transmute_copy(&simplificationoption), ::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance), ::core::mem::transmute(&geometrysink)).into()
        }
        unsafe extern "system" fn Tessellate<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, tessellationsink: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Tessellate(::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance), ::core::mem::transmute(&tessellationsink)).into()
        }
        unsafe extern "system" fn CombineWithGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inputgeometry: ::windows::core::RawPtr, combinemode: D2D1_COMBINE_MODE, inputgeometrytransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, geometrysink: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).CombineWithGeometry(::core::mem::transmute(&inputgeometry), ::core::mem::transmute_copy(&combinemode), ::core::mem::transmute_copy(&inputgeometrytransform), ::core::mem::transmute_copy(&flatteningtolerance), ::core::mem::transmute(&geometrysink)).into()
        }
        unsafe extern "system" fn Outline<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, geometrysink: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Outline(::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance), ::core::mem::transmute(&geometrysink)).into()
        }
        unsafe extern "system" fn ComputeArea<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, area: *mut f32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).ComputeArea(::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance)) {
                ::core::result::Result::Ok(ok__) => {
                    *area = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn ComputeLength<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, length: *mut f32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).ComputeLength(::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance)) {
                ::core::result::Result::Ok(ok__) => {
                    *length = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn ComputePointAtLength<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, length: f32, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, point: *mut Common::D2D_POINT_2F, unittangentvector: *mut Common::D2D_POINT_2F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ComputePointAtLength(::core::mem::transmute_copy(&length), ::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance), ::core::mem::transmute_copy(&point), ::core::mem::transmute_copy(&unittangentvector)).into()
        }
        unsafe extern "system" fn Widen<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Geometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, strokewidth: f32, strokestyle: ::windows::core::RawPtr, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, geometrysink: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Widen(::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle), ::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance), ::core::mem::transmute(&geometrysink)).into()
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetBounds: GetBounds::<Identity, Impl, OFFSET>,
            GetWidenedBounds: GetWidenedBounds::<Identity, Impl, OFFSET>,
            StrokeContainsPoint: StrokeContainsPoint::<Identity, Impl, OFFSET>,
            FillContainsPoint: FillContainsPoint::<Identity, Impl, OFFSET>,
            CompareWithGeometry: CompareWithGeometry::<Identity, Impl, OFFSET>,
            Simplify: Simplify::<Identity, Impl, OFFSET>,
            Tessellate: Tessellate::<Identity, Impl, OFFSET>,
            CombineWithGeometry: CombineWithGeometry::<Identity, Impl, OFFSET>,
            Outline: Outline::<Identity, Impl, OFFSET>,
            ComputeArea: ComputeArea::<Identity, Impl, OFFSET>,
            ComputeLength: ComputeLength::<Identity, Impl, OFFSET>,
            ComputePointAtLength: ComputePointAtLength::<Identity, Impl, OFFSET>,
            Widen: Widen::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Geometry as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1GeometryGroup_Impl: Sized + ID2D1Resource_Impl + ID2D1Geometry_Impl {
    fn GetFillMode(&mut self) -> Common::D2D1_FILL_MODE;
    fn GetSourceGeometryCount(&mut self) -> u32;
    fn GetSourceGeometries(&mut self, geometries: *mut ::core::option::Option<ID2D1Geometry>, geometriescount: u32);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1GeometryGroup_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometryGroup_Impl, const OFFSET: isize>() -> ID2D1GeometryGroup_Vtbl {
        unsafe extern "system" fn GetFillMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometryGroup_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> Common::D2D1_FILL_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetFillMode()
        }
        unsafe extern "system" fn GetSourceGeometryCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometryGroup_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSourceGeometryCount()
        }
        unsafe extern "system" fn GetSourceGeometries<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometryGroup_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, geometries: *mut ::windows::core::RawPtr, geometriescount: u32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSourceGeometries(::core::mem::transmute_copy(&geometries), ::core::mem::transmute_copy(&geometriescount))
        }
        Self {
            base: ID2D1Geometry_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetFillMode: GetFillMode::<Identity, Impl, OFFSET>,
            GetSourceGeometryCount: GetSourceGeometryCount::<Identity, Impl, OFFSET>,
            GetSourceGeometries: GetSourceGeometries::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GeometryGroup as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Geometry as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1GeometryRealization_Impl: Sized + ID2D1Resource_Impl {}
impl ID2D1GeometryRealization_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometryRealization_Impl, const OFFSET: isize>() -> ID2D1GeometryRealization_Vtbl {
        Self { base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>() }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GeometryRealization as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
pub trait ID2D1GeometrySink_Impl: Sized + Common::ID2D1SimplifiedGeometrySink_Impl {
    fn AddLine(&mut self, point: &Common::D2D_POINT_2F);
    fn AddBezier(&mut self, bezier: *const Common::D2D1_BEZIER_SEGMENT);
    fn AddQuadraticBezier(&mut self, bezier: *const D2D1_QUADRATIC_BEZIER_SEGMENT);
    fn AddQuadraticBeziers(&mut self, beziers: *const D2D1_QUADRATIC_BEZIER_SEGMENT, bezierscount: u32);
    fn AddArc(&mut self, arc: *const D2D1_ARC_SEGMENT);
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
impl ID2D1GeometrySink_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometrySink_Impl, const OFFSET: isize>() -> ID2D1GeometrySink_Vtbl {
        unsafe extern "system" fn AddLine<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometrySink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, point: Common::D2D_POINT_2F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AddLine(::core::mem::transmute_copy(&point))
        }
        unsafe extern "system" fn AddBezier<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometrySink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bezier: *const Common::D2D1_BEZIER_SEGMENT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AddBezier(::core::mem::transmute_copy(&bezier))
        }
        unsafe extern "system" fn AddQuadraticBezier<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometrySink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bezier: *const D2D1_QUADRATIC_BEZIER_SEGMENT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AddQuadraticBezier(::core::mem::transmute_copy(&bezier))
        }
        unsafe extern "system" fn AddQuadraticBeziers<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometrySink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, beziers: *const D2D1_QUADRATIC_BEZIER_SEGMENT, bezierscount: u32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AddQuadraticBeziers(::core::mem::transmute_copy(&beziers), ::core::mem::transmute_copy(&bezierscount))
        }
        unsafe extern "system" fn AddArc<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GeometrySink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, arc: *const D2D1_ARC_SEGMENT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AddArc(::core::mem::transmute_copy(&arc))
        }
        Self {
            base: Common::ID2D1SimplifiedGeometrySink_Vtbl::new::<Identity, Impl, OFFSET>(),
            AddLine: AddLine::<Identity, Impl, OFFSET>,
            AddBezier: AddBezier::<Identity, Impl, OFFSET>,
            AddQuadraticBezier: AddQuadraticBezier::<Identity, Impl, OFFSET>,
            AddQuadraticBeziers: AddQuadraticBeziers::<Identity, Impl, OFFSET>,
            AddArc: AddArc::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GeometrySink as ::windows::core::Interface>::IID || iid == &<Common::ID2D1SimplifiedGeometrySink as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
pub trait ID2D1GradientMesh_Impl: Sized + ID2D1Resource_Impl {
    fn GetPatchCount(&mut self) -> u32;
    fn GetPatches(&mut self, startindex: u32, patches: *mut D2D1_GRADIENT_MESH_PATCH, patchescount: u32) -> ::windows::core::Result<()>;
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
impl ID2D1GradientMesh_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientMesh_Impl, const OFFSET: isize>() -> ID2D1GradientMesh_Vtbl {
        unsafe extern "system" fn GetPatchCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientMesh_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPatchCount()
        }
        unsafe extern "system" fn GetPatches<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientMesh_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, startindex: u32, patches: *mut D2D1_GRADIENT_MESH_PATCH, patchescount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPatches(::core::mem::transmute_copy(&startindex), ::core::mem::transmute_copy(&patches), ::core::mem::transmute_copy(&patchescount)).into()
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetPatchCount: GetPatchCount::<Identity, Impl, OFFSET>,
            GetPatches: GetPatches::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GradientMesh as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
pub trait ID2D1GradientStopCollection_Impl: Sized + ID2D1Resource_Impl {
    fn GetGradientStopCount(&mut self) -> u32;
    fn GetGradientStops(&mut self, gradientstops: *mut D2D1_GRADIENT_STOP, gradientstopscount: u32);
    fn GetColorInterpolationGamma(&mut self) -> D2D1_GAMMA;
    fn GetExtendMode(&mut self) -> D2D1_EXTEND_MODE;
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
impl ID2D1GradientStopCollection_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection_Impl, const OFFSET: isize>() -> ID2D1GradientStopCollection_Vtbl {
        unsafe extern "system" fn GetGradientStopCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetGradientStopCount()
        }
        unsafe extern "system" fn GetGradientStops<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gradientstops: *mut D2D1_GRADIENT_STOP, gradientstopscount: u32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetGradientStops(::core::mem::transmute_copy(&gradientstops), ::core::mem::transmute_copy(&gradientstopscount))
        }
        unsafe extern "system" fn GetColorInterpolationGamma<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_GAMMA {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetColorInterpolationGamma()
        }
        unsafe extern "system" fn GetExtendMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_EXTEND_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetExtendMode()
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetGradientStopCount: GetGradientStopCount::<Identity, Impl, OFFSET>,
            GetGradientStops: GetGradientStops::<Identity, Impl, OFFSET>,
            GetColorInterpolationGamma: GetColorInterpolationGamma::<Identity, Impl, OFFSET>,
            GetExtendMode: GetExtendMode::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GradientStopCollection as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
pub trait ID2D1GradientStopCollection1_Impl: Sized + ID2D1Resource_Impl + ID2D1GradientStopCollection_Impl {
    fn GetGradientStops1(&mut self, gradientstops: *mut D2D1_GRADIENT_STOP, gradientstopscount: u32);
    fn GetPreInterpolationSpace(&mut self) -> D2D1_COLOR_SPACE;
    fn GetPostInterpolationSpace(&mut self) -> D2D1_COLOR_SPACE;
    fn GetBufferPrecision(&mut self) -> D2D1_BUFFER_PRECISION;
    fn GetColorInterpolationMode(&mut self) -> D2D1_COLOR_INTERPOLATION_MODE;
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
impl ID2D1GradientStopCollection1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection1_Impl, const OFFSET: isize>() -> ID2D1GradientStopCollection1_Vtbl {
        unsafe extern "system" fn GetGradientStops1<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gradientstops: *mut D2D1_GRADIENT_STOP, gradientstopscount: u32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetGradientStops1(::core::mem::transmute_copy(&gradientstops), ::core::mem::transmute_copy(&gradientstopscount))
        }
        unsafe extern "system" fn GetPreInterpolationSpace<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_COLOR_SPACE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPreInterpolationSpace()
        }
        unsafe extern "system" fn GetPostInterpolationSpace<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_COLOR_SPACE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPostInterpolationSpace()
        }
        unsafe extern "system" fn GetBufferPrecision<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_BUFFER_PRECISION {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetBufferPrecision()
        }
        unsafe extern "system" fn GetColorInterpolationMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1GradientStopCollection1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_COLOR_INTERPOLATION_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetColorInterpolationMode()
        }
        Self {
            base: ID2D1GradientStopCollection_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetGradientStops1: GetGradientStops1::<Identity, Impl, OFFSET>,
            GetPreInterpolationSpace: GetPreInterpolationSpace::<Identity, Impl, OFFSET>,
            GetPostInterpolationSpace: GetPostInterpolationSpace::<Identity, Impl, OFFSET>,
            GetBufferPrecision: GetBufferPrecision::<Identity, Impl, OFFSET>,
            GetColorInterpolationMode: GetColorInterpolationMode::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1GradientStopCollection1 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1GradientStopCollection as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1HwndRenderTarget_Impl: Sized + ID2D1Resource_Impl + ID2D1RenderTarget_Impl {
    fn CheckWindowState(&mut self) -> D2D1_WINDOW_STATE;
    fn Resize(&mut self, pixelsize: *const Common::D2D_SIZE_U) -> ::windows::core::Result<()>;
    fn GetHwnd(&mut self) -> super::super::Foundation::HWND;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1HwndRenderTarget_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1HwndRenderTarget_Impl, const OFFSET: isize>() -> ID2D1HwndRenderTarget_Vtbl {
        unsafe extern "system" fn CheckWindowState<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1HwndRenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_WINDOW_STATE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).CheckWindowState()
        }
        unsafe extern "system" fn Resize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1HwndRenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, pixelsize: *const Common::D2D_SIZE_U) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Resize(::core::mem::transmute_copy(&pixelsize)).into()
        }
        unsafe extern "system" fn GetHwnd<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1HwndRenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> super::super::Foundation::HWND {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetHwnd()
        }
        Self {
            base: ID2D1RenderTarget_Vtbl::new::<Identity, Impl, OFFSET>(),
            CheckWindowState: CheckWindowState::<Identity, Impl, OFFSET>,
            Resize: Resize::<Identity, Impl, OFFSET>,
            GetHwnd: GetHwnd::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1HwndRenderTarget as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1Image_Impl: Sized + ID2D1Resource_Impl {}
impl ID2D1Image_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Image_Impl, const OFFSET: isize>() -> ID2D1Image_Vtbl {
        Self { base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>() }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Image as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1ImageBrush_Impl: Sized + ID2D1Resource_Impl + ID2D1Brush_Impl {
    fn SetImage(&mut self, image: &::core::option::Option<ID2D1Image>);
    fn SetExtendModeX(&mut self, extendmodex: D2D1_EXTEND_MODE);
    fn SetExtendModeY(&mut self, extendmodey: D2D1_EXTEND_MODE);
    fn SetInterpolationMode(&mut self, interpolationmode: D2D1_INTERPOLATION_MODE);
    fn SetSourceRectangle(&mut self, sourcerectangle: *const Common::D2D_RECT_F);
    fn GetImage(&mut self, image: *mut ::core::option::Option<ID2D1Image>);
    fn GetExtendModeX(&mut self) -> D2D1_EXTEND_MODE;
    fn GetExtendModeY(&mut self) -> D2D1_EXTEND_MODE;
    fn GetInterpolationMode(&mut self) -> D2D1_INTERPOLATION_MODE;
    fn GetSourceRectangle(&mut self, sourcerectangle: *mut Common::D2D_RECT_F);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1ImageBrush_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>() -> ID2D1ImageBrush_Vtbl {
        unsafe extern "system" fn SetImage<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetImage(::core::mem::transmute(&image))
        }
        unsafe extern "system" fn SetExtendModeX<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, extendmodex: D2D1_EXTEND_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetExtendModeX(::core::mem::transmute_copy(&extendmodex))
        }
        unsafe extern "system" fn SetExtendModeY<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, extendmodey: D2D1_EXTEND_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetExtendModeY(::core::mem::transmute_copy(&extendmodey))
        }
        unsafe extern "system" fn SetInterpolationMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, interpolationmode: D2D1_INTERPOLATION_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetInterpolationMode(::core::mem::transmute_copy(&interpolationmode))
        }
        unsafe extern "system" fn SetSourceRectangle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, sourcerectangle: *const Common::D2D_RECT_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetSourceRectangle(::core::mem::transmute_copy(&sourcerectangle))
        }
        unsafe extern "system" fn GetImage<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, image: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetImage(::core::mem::transmute_copy(&image))
        }
        unsafe extern "system" fn GetExtendModeX<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_EXTEND_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetExtendModeX()
        }
        unsafe extern "system" fn GetExtendModeY<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_EXTEND_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetExtendModeY()
        }
        unsafe extern "system" fn GetInterpolationMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_INTERPOLATION_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetInterpolationMode()
        }
        unsafe extern "system" fn GetSourceRectangle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, sourcerectangle: *mut Common::D2D_RECT_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSourceRectangle(::core::mem::transmute_copy(&sourcerectangle))
        }
        Self {
            base: ID2D1Brush_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetImage: SetImage::<Identity, Impl, OFFSET>,
            SetExtendModeX: SetExtendModeX::<Identity, Impl, OFFSET>,
            SetExtendModeY: SetExtendModeY::<Identity, Impl, OFFSET>,
            SetInterpolationMode: SetInterpolationMode::<Identity, Impl, OFFSET>,
            SetSourceRectangle: SetSourceRectangle::<Identity, Impl, OFFSET>,
            GetImage: GetImage::<Identity, Impl, OFFSET>,
            GetExtendModeX: GetExtendModeX::<Identity, Impl, OFFSET>,
            GetExtendModeY: GetExtendModeY::<Identity, Impl, OFFSET>,
            GetInterpolationMode: GetInterpolationMode::<Identity, Impl, OFFSET>,
            GetSourceRectangle: GetSourceRectangle::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1ImageBrush as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Brush as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1ImageSource_Impl: Sized + ID2D1Resource_Impl + ID2D1Image_Impl {
    fn OfferResources(&mut self) -> ::windows::core::Result<()>;
    fn TryReclaimResources(&mut self) -> ::windows::core::Result<super::super::Foundation::BOOL>;
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1ImageSource_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageSource_Impl, const OFFSET: isize>() -> ID2D1ImageSource_Vtbl {
        unsafe extern "system" fn OfferResources<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageSource_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).OfferResources().into()
        }
        unsafe extern "system" fn TryReclaimResources<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageSource_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, resourcesdiscarded: *mut super::super::Foundation::BOOL) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).TryReclaimResources() {
                ::core::result::Result::Ok(ok__) => {
                    *resourcesdiscarded = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1Image_Vtbl::new::<Identity, Impl, OFFSET>(),
            OfferResources: OfferResources::<Identity, Impl, OFFSET>,
            TryReclaimResources: TryReclaimResources::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1ImageSource as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Image as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1ImageSourceFromWic_Impl: Sized + ID2D1Resource_Impl + ID2D1Image_Impl + ID2D1ImageSource_Impl {
    fn EnsureCached(&mut self, rectangletofill: *const Common::D2D_RECT_U) -> ::windows::core::Result<()>;
    fn TrimCache(&mut self, rectangletopreserve: *const Common::D2D_RECT_U) -> ::windows::core::Result<()>;
    fn GetSource(&mut self, wicbitmapsource: *mut ::core::option::Option<super::Imaging::IWICBitmapSource>);
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1ImageSourceFromWic_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageSourceFromWic_Impl, const OFFSET: isize>() -> ID2D1ImageSourceFromWic_Vtbl {
        unsafe extern "system" fn EnsureCached<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageSourceFromWic_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rectangletofill: *const Common::D2D_RECT_U) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).EnsureCached(::core::mem::transmute_copy(&rectangletofill)).into()
        }
        unsafe extern "system" fn TrimCache<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageSourceFromWic_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rectangletopreserve: *const Common::D2D_RECT_U) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).TrimCache(::core::mem::transmute_copy(&rectangletopreserve)).into()
        }
        unsafe extern "system" fn GetSource<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ImageSourceFromWic_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, wicbitmapsource: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSource(::core::mem::transmute_copy(&wicbitmapsource))
        }
        Self {
            base: ID2D1ImageSource_Vtbl::new::<Identity, Impl, OFFSET>(),
            EnsureCached: EnsureCached::<Identity, Impl, OFFSET>,
            TrimCache: TrimCache::<Identity, Impl, OFFSET>,
            GetSource: GetSource::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1ImageSourceFromWic as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Image as ::windows::core::Interface>::IID || iid == &<ID2D1ImageSource as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1Ink_Impl: Sized + ID2D1Resource_Impl {
    fn SetStartPoint(&mut self, startpoint: *const D2D1_INK_POINT);
    fn GetStartPoint(&mut self) -> D2D1_INK_POINT;
    fn AddSegments(&mut self, segments: *const D2D1_INK_BEZIER_SEGMENT, segmentscount: u32) -> ::windows::core::Result<()>;
    fn RemoveSegmentsAtEnd(&mut self, segmentscount: u32) -> ::windows::core::Result<()>;
    fn SetSegments(&mut self, startsegment: u32, segments: *const D2D1_INK_BEZIER_SEGMENT, segmentscount: u32) -> ::windows::core::Result<()>;
    fn SetSegmentAtEnd(&mut self, segment: *const D2D1_INK_BEZIER_SEGMENT) -> ::windows::core::Result<()>;
    fn GetSegmentCount(&mut self) -> u32;
    fn GetSegments(&mut self, startsegment: u32, segments: *mut D2D1_INK_BEZIER_SEGMENT, segmentscount: u32) -> ::windows::core::Result<()>;
    fn StreamAsGeometry(&mut self, inkstyle: &::core::option::Option<ID2D1InkStyle>, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, geometrysink: &::core::option::Option<Common::ID2D1SimplifiedGeometrySink>) -> ::windows::core::Result<()>;
    fn GetBounds(&mut self, inkstyle: &::core::option::Option<ID2D1InkStyle>, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2) -> ::windows::core::Result<Common::D2D_RECT_F>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1Ink_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>() -> ID2D1Ink_Vtbl {
        unsafe extern "system" fn SetStartPoint<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, startpoint: *const D2D1_INK_POINT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetStartPoint(::core::mem::transmute_copy(&startpoint))
        }
        unsafe extern "system" fn GetStartPoint<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut D2D1_INK_POINT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetStartPoint()
        }
        unsafe extern "system" fn AddSegments<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, segments: *const D2D1_INK_BEZIER_SEGMENT, segmentscount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AddSegments(::core::mem::transmute_copy(&segments), ::core::mem::transmute_copy(&segmentscount)).into()
        }
        unsafe extern "system" fn RemoveSegmentsAtEnd<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, segmentscount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RemoveSegmentsAtEnd(::core::mem::transmute_copy(&segmentscount)).into()
        }
        unsafe extern "system" fn SetSegments<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, startsegment: u32, segments: *const D2D1_INK_BEZIER_SEGMENT, segmentscount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetSegments(::core::mem::transmute_copy(&startsegment), ::core::mem::transmute_copy(&segments), ::core::mem::transmute_copy(&segmentscount)).into()
        }
        unsafe extern "system" fn SetSegmentAtEnd<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, segment: *const D2D1_INK_BEZIER_SEGMENT) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetSegmentAtEnd(::core::mem::transmute_copy(&segment)).into()
        }
        unsafe extern "system" fn GetSegmentCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSegmentCount()
        }
        unsafe extern "system" fn GetSegments<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, startsegment: u32, segments: *mut D2D1_INK_BEZIER_SEGMENT, segmentscount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSegments(::core::mem::transmute_copy(&startsegment), ::core::mem::transmute_copy(&segments), ::core::mem::transmute_copy(&segmentscount)).into()
        }
        unsafe extern "system" fn StreamAsGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inkstyle: ::windows::core::RawPtr, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, geometrysink: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).StreamAsGeometry(::core::mem::transmute(&inkstyle), ::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance), ::core::mem::transmute(&geometrysink)).into()
        }
        unsafe extern "system" fn GetBounds<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Ink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inkstyle: ::windows::core::RawPtr, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, bounds: *mut Common::D2D_RECT_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetBounds(::core::mem::transmute(&inkstyle), ::core::mem::transmute_copy(&worldtransform)) {
                ::core::result::Result::Ok(ok__) => {
                    *bounds = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetStartPoint: SetStartPoint::<Identity, Impl, OFFSET>,
            GetStartPoint: GetStartPoint::<Identity, Impl, OFFSET>,
            AddSegments: AddSegments::<Identity, Impl, OFFSET>,
            RemoveSegmentsAtEnd: RemoveSegmentsAtEnd::<Identity, Impl, OFFSET>,
            SetSegments: SetSegments::<Identity, Impl, OFFSET>,
            SetSegmentAtEnd: SetSegmentAtEnd::<Identity, Impl, OFFSET>,
            GetSegmentCount: GetSegmentCount::<Identity, Impl, OFFSET>,
            GetSegments: GetSegments::<Identity, Impl, OFFSET>,
            StreamAsGeometry: StreamAsGeometry::<Identity, Impl, OFFSET>,
            GetBounds: GetBounds::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Ink as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Foundation_Numerics")]
pub trait ID2D1InkStyle_Impl: Sized + ID2D1Resource_Impl {
    fn SetNibTransform(&mut self, transform: *const super::super::super::Foundation::Numerics::Matrix3x2);
    fn GetNibTransform(&mut self, transform: *mut super::super::super::Foundation::Numerics::Matrix3x2);
    fn SetNibShape(&mut self, nibshape: D2D1_INK_NIB_SHAPE);
    fn GetNibShape(&mut self) -> D2D1_INK_NIB_SHAPE;
}
#[cfg(feature = "Foundation_Numerics")]
impl ID2D1InkStyle_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1InkStyle_Impl, const OFFSET: isize>() -> ID2D1InkStyle_Vtbl {
        unsafe extern "system" fn SetNibTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1InkStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, transform: *const super::super::super::Foundation::Numerics::Matrix3x2) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetNibTransform(::core::mem::transmute_copy(&transform))
        }
        unsafe extern "system" fn GetNibTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1InkStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, transform: *mut super::super::super::Foundation::Numerics::Matrix3x2) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetNibTransform(::core::mem::transmute_copy(&transform))
        }
        unsafe extern "system" fn SetNibShape<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1InkStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, nibshape: D2D1_INK_NIB_SHAPE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetNibShape(::core::mem::transmute_copy(&nibshape))
        }
        unsafe extern "system" fn GetNibShape<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1InkStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_INK_NIB_SHAPE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetNibShape()
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetNibTransform: SetNibTransform::<Identity, Impl, OFFSET>,
            GetNibTransform: GetNibTransform::<Identity, Impl, OFFSET>,
            SetNibShape: SetNibShape::<Identity, Impl, OFFSET>,
            GetNibShape: GetNibShape::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1InkStyle as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
pub trait ID2D1Layer_Impl: Sized + ID2D1Resource_Impl {
    fn GetSize(&mut self) -> Common::D2D_SIZE_F;
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
impl ID2D1Layer_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Layer_Impl, const OFFSET: isize>() -> ID2D1Layer_Vtbl {
        unsafe extern "system" fn GetSize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Layer_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D_SIZE_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetSize()
        }
        Self { base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(), GetSize: GetSize::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Layer as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1LinearGradientBrush_Impl: Sized + ID2D1Resource_Impl + ID2D1Brush_Impl {
    fn SetStartPoint(&mut self, startpoint: &Common::D2D_POINT_2F);
    fn SetEndPoint(&mut self, endpoint: &Common::D2D_POINT_2F);
    fn GetStartPoint(&mut self) -> Common::D2D_POINT_2F;
    fn GetEndPoint(&mut self) -> Common::D2D_POINT_2F;
    fn GetGradientStopCollection(&mut self, gradientstopcollection: *mut ::core::option::Option<ID2D1GradientStopCollection>);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1LinearGradientBrush_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1LinearGradientBrush_Impl, const OFFSET: isize>() -> ID2D1LinearGradientBrush_Vtbl {
        unsafe extern "system" fn SetStartPoint<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1LinearGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, startpoint: Common::D2D_POINT_2F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetStartPoint(::core::mem::transmute_copy(&startpoint))
        }
        unsafe extern "system" fn SetEndPoint<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1LinearGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, endpoint: Common::D2D_POINT_2F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetEndPoint(::core::mem::transmute_copy(&endpoint))
        }
        unsafe extern "system" fn GetStartPoint<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1LinearGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D_POINT_2F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetStartPoint()
        }
        unsafe extern "system" fn GetEndPoint<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1LinearGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D_POINT_2F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetEndPoint()
        }
        unsafe extern "system" fn GetGradientStopCollection<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1LinearGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gradientstopcollection: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetGradientStopCollection(::core::mem::transmute_copy(&gradientstopcollection))
        }
        Self {
            base: ID2D1Brush_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetStartPoint: SetStartPoint::<Identity, Impl, OFFSET>,
            SetEndPoint: SetEndPoint::<Identity, Impl, OFFSET>,
            GetStartPoint: GetStartPoint::<Identity, Impl, OFFSET>,
            GetEndPoint: GetEndPoint::<Identity, Impl, OFFSET>,
            GetGradientStopCollection: GetGradientStopCollection::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1LinearGradientBrush as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Brush as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1LookupTable3D_Impl: Sized + ID2D1Resource_Impl {}
impl ID2D1LookupTable3D_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1LookupTable3D_Impl, const OFFSET: isize>() -> ID2D1LookupTable3D_Vtbl {
        Self { base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>() }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1LookupTable3D as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1Mesh_Impl: Sized + ID2D1Resource_Impl {
    fn Open(&mut self) -> ::windows::core::Result<ID2D1TessellationSink>;
}
impl ID2D1Mesh_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Mesh_Impl, const OFFSET: isize>() -> ID2D1Mesh_Vtbl {
        unsafe extern "system" fn Open<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Mesh_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, tessellationsink: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).Open() {
                ::core::result::Result::Ok(ok__) => {
                    *tessellationsink = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self { base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(), Open: Open::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Mesh as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1Multithread_Impl: Sized {
    fn GetMultithreadProtected(&mut self) -> super::super::Foundation::BOOL;
    fn Enter(&mut self);
    fn Leave(&mut self);
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1Multithread_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Multithread_Impl, const OFFSET: isize>() -> ID2D1Multithread_Vtbl {
        unsafe extern "system" fn GetMultithreadProtected<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Multithread_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetMultithreadProtected()
        }
        unsafe extern "system" fn Enter<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Multithread_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Enter()
        }
        unsafe extern "system" fn Leave<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Multithread_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Leave()
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            GetMultithreadProtected: GetMultithreadProtected::<Identity, Impl, OFFSET>,
            Enter: Enter::<Identity, Impl, OFFSET>,
            Leave: Leave::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Multithread as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1OffsetTransform_Impl: Sized + ID2D1TransformNode_Impl {
    fn SetOffset(&mut self, offset: &super::super::Foundation::POINT);
    fn GetOffset(&mut self) -> super::super::Foundation::POINT;
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1OffsetTransform_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1OffsetTransform_Impl, const OFFSET: isize>() -> ID2D1OffsetTransform_Vtbl {
        unsafe extern "system" fn SetOffset<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1OffsetTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, offset: super::super::Foundation::POINT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetOffset(::core::mem::transmute_copy(&offset))
        }
        unsafe extern "system" fn GetOffset<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1OffsetTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut super::super::Foundation::POINT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetOffset()
        }
        Self {
            base: ID2D1TransformNode_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetOffset: SetOffset::<Identity, Impl, OFFSET>,
            GetOffset: GetOffset::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1OffsetTransform as ::windows::core::Interface>::IID || iid == &<ID2D1TransformNode as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1PathGeometry_Impl: Sized + ID2D1Resource_Impl + ID2D1Geometry_Impl {
    fn Open(&mut self) -> ::windows::core::Result<ID2D1GeometrySink>;
    fn Stream(&mut self, geometrysink: &::core::option::Option<ID2D1GeometrySink>) -> ::windows::core::Result<()>;
    fn GetSegmentCount(&mut self) -> ::windows::core::Result<u32>;
    fn GetFigureCount(&mut self) -> ::windows::core::Result<u32>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1PathGeometry_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1PathGeometry_Impl, const OFFSET: isize>() -> ID2D1PathGeometry_Vtbl {
        unsafe extern "system" fn Open<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1PathGeometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, geometrysink: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).Open() {
                ::core::result::Result::Ok(ok__) => {
                    *geometrysink = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn Stream<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1PathGeometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, geometrysink: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Stream(::core::mem::transmute(&geometrysink)).into()
        }
        unsafe extern "system" fn GetSegmentCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1PathGeometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, count: *mut u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetSegmentCount() {
                ::core::result::Result::Ok(ok__) => {
                    *count = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn GetFigureCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1PathGeometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, count: *mut u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetFigureCount() {
                ::core::result::Result::Ok(ok__) => {
                    *count = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1Geometry_Vtbl::new::<Identity, Impl, OFFSET>(),
            Open: Open::<Identity, Impl, OFFSET>,
            Stream: Stream::<Identity, Impl, OFFSET>,
            GetSegmentCount: GetSegmentCount::<Identity, Impl, OFFSET>,
            GetFigureCount: GetFigureCount::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1PathGeometry as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Geometry as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1PathGeometry1_Impl: Sized + ID2D1Resource_Impl + ID2D1Geometry_Impl + ID2D1PathGeometry_Impl {
    fn ComputePointAndSegmentAtLength(&mut self, length: f32, startsegment: u32, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32) -> ::windows::core::Result<D2D1_POINT_DESCRIPTION>;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1PathGeometry1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1PathGeometry1_Impl, const OFFSET: isize>() -> ID2D1PathGeometry1_Vtbl {
        unsafe extern "system" fn ComputePointAndSegmentAtLength<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1PathGeometry1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, length: f32, startsegment: u32, worldtransform: *const super::super::super::Foundation::Numerics::Matrix3x2, flatteningtolerance: f32, pointdescription: *mut D2D1_POINT_DESCRIPTION) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).ComputePointAndSegmentAtLength(::core::mem::transmute_copy(&length), ::core::mem::transmute_copy(&startsegment), ::core::mem::transmute_copy(&worldtransform), ::core::mem::transmute_copy(&flatteningtolerance)) {
                ::core::result::Result::Ok(ok__) => {
                    *pointdescription = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1PathGeometry_Vtbl::new::<Identity, Impl, OFFSET>(),
            ComputePointAndSegmentAtLength: ComputePointAndSegmentAtLength::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1PathGeometry1 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Geometry as ::windows::core::Interface>::IID || iid == &<ID2D1PathGeometry as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_System_Com"))]
pub trait ID2D1PrintControl_Impl: Sized {
    fn AddPage(&mut self, commandlist: &::core::option::Option<ID2D1CommandList>, pagesize: &Common::D2D_SIZE_F, pageprintticketstream: &::core::option::Option<super::super::System::Com::IStream>, tag1: *mut u64, tag2: *mut u64) -> ::windows::core::Result<()>;
    fn Close(&mut self) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_System_Com"))]
impl ID2D1PrintControl_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1PrintControl_Impl, const OFFSET: isize>() -> ID2D1PrintControl_Vtbl {
        unsafe extern "system" fn AddPage<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1PrintControl_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, commandlist: ::windows::core::RawPtr, pagesize: Common::D2D_SIZE_F, pageprintticketstream: ::windows::core::RawPtr, tag1: *mut u64, tag2: *mut u64) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AddPage(::core::mem::transmute(&commandlist), ::core::mem::transmute_copy(&pagesize), ::core::mem::transmute(&pageprintticketstream), ::core::mem::transmute_copy(&tag1), ::core::mem::transmute_copy(&tag2)).into()
        }
        unsafe extern "system" fn Close<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1PrintControl_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Close().into()
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            AddPage: AddPage::<Identity, Impl, OFFSET>,
            Close: Close::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1PrintControl as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1Properties_Impl: Sized {
    fn GetPropertyCount(&mut self) -> u32;
    fn GetPropertyName(&mut self, index: u32, name: super::super::Foundation::PWSTR, namecount: u32) -> ::windows::core::Result<()>;
    fn GetPropertyNameLength(&mut self, index: u32) -> u32;
    fn GetType(&mut self, index: u32) -> D2D1_PROPERTY_TYPE;
    fn GetPropertyIndex(&mut self, name: super::super::Foundation::PWSTR) -> u32;
    fn SetValueByName(&mut self, name: super::super::Foundation::PWSTR, r#type: D2D1_PROPERTY_TYPE, data: *const u8, datasize: u32) -> ::windows::core::Result<()>;
    fn SetValue(&mut self, index: u32, r#type: D2D1_PROPERTY_TYPE, data: *const u8, datasize: u32) -> ::windows::core::Result<()>;
    fn GetValueByName(&mut self, name: super::super::Foundation::PWSTR, r#type: D2D1_PROPERTY_TYPE, data: *mut u8, datasize: u32) -> ::windows::core::Result<()>;
    fn GetValue(&mut self, index: u32, r#type: D2D1_PROPERTY_TYPE, data: *mut u8, datasize: u32) -> ::windows::core::Result<()>;
    fn GetValueSize(&mut self, index: u32) -> u32;
    fn GetSubProperties(&mut self, index: u32) -> ::windows::core::Result<ID2D1Properties>;
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1Properties_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>() -> ID2D1Properties_Vtbl {
        unsafe extern "system" fn GetPropertyCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPropertyCount()
        }
        unsafe extern "system" fn GetPropertyName<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32, name: super::super::Foundation::PWSTR, namecount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPropertyName(::core::mem::transmute_copy(&index), ::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&namecount)).into()
        }
        unsafe extern "system" fn GetPropertyNameLength<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPropertyNameLength(::core::mem::transmute_copy(&index))
        }
        unsafe extern "system" fn GetType<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32) -> D2D1_PROPERTY_TYPE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetType(::core::mem::transmute_copy(&index))
        }
        unsafe extern "system" fn GetPropertyIndex<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPropertyIndex(::core::mem::transmute_copy(&name))
        }
        unsafe extern "system" fn SetValueByName<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, r#type: D2D1_PROPERTY_TYPE, data: *const u8, datasize: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetValueByName(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&r#type), ::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&datasize)).into()
        }
        unsafe extern "system" fn SetValue<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32, r#type: D2D1_PROPERTY_TYPE, data: *const u8, datasize: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetValue(::core::mem::transmute_copy(&index), ::core::mem::transmute_copy(&r#type), ::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&datasize)).into()
        }
        unsafe extern "system" fn GetValueByName<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, r#type: D2D1_PROPERTY_TYPE, data: *mut u8, datasize: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetValueByName(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&r#type), ::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&datasize)).into()
        }
        unsafe extern "system" fn GetValue<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32, r#type: D2D1_PROPERTY_TYPE, data: *mut u8, datasize: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetValue(::core::mem::transmute_copy(&index), ::core::mem::transmute_copy(&r#type), ::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&datasize)).into()
        }
        unsafe extern "system" fn GetValueSize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetValueSize(::core::mem::transmute_copy(&index))
        }
        unsafe extern "system" fn GetSubProperties<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Properties_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32, subproperties: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetSubProperties(::core::mem::transmute_copy(&index)) {
                ::core::result::Result::Ok(ok__) => {
                    *subproperties = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            GetPropertyCount: GetPropertyCount::<Identity, Impl, OFFSET>,
            GetPropertyName: GetPropertyName::<Identity, Impl, OFFSET>,
            GetPropertyNameLength: GetPropertyNameLength::<Identity, Impl, OFFSET>,
            GetType: GetType::<Identity, Impl, OFFSET>,
            GetPropertyIndex: GetPropertyIndex::<Identity, Impl, OFFSET>,
            SetValueByName: SetValueByName::<Identity, Impl, OFFSET>,
            SetValue: SetValue::<Identity, Impl, OFFSET>,
            GetValueByName: GetValueByName::<Identity, Impl, OFFSET>,
            GetValue: GetValue::<Identity, Impl, OFFSET>,
            GetValueSize: GetValueSize::<Identity, Impl, OFFSET>,
            GetSubProperties: GetSubProperties::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Properties as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1RadialGradientBrush_Impl: Sized + ID2D1Resource_Impl + ID2D1Brush_Impl {
    fn SetCenter(&mut self, center: &Common::D2D_POINT_2F);
    fn SetGradientOriginOffset(&mut self, gradientoriginoffset: &Common::D2D_POINT_2F);
    fn SetRadiusX(&mut self, radiusx: f32);
    fn SetRadiusY(&mut self, radiusy: f32);
    fn GetCenter(&mut self) -> Common::D2D_POINT_2F;
    fn GetGradientOriginOffset(&mut self) -> Common::D2D_POINT_2F;
    fn GetRadiusX(&mut self) -> f32;
    fn GetRadiusY(&mut self) -> f32;
    fn GetGradientStopCollection(&mut self, gradientstopcollection: *mut ::core::option::Option<ID2D1GradientStopCollection>);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1RadialGradientBrush_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RadialGradientBrush_Impl, const OFFSET: isize>() -> ID2D1RadialGradientBrush_Vtbl {
        unsafe extern "system" fn SetCenter<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RadialGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, center: Common::D2D_POINT_2F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetCenter(::core::mem::transmute_copy(&center))
        }
        unsafe extern "system" fn SetGradientOriginOffset<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RadialGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gradientoriginoffset: Common::D2D_POINT_2F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetGradientOriginOffset(::core::mem::transmute_copy(&gradientoriginoffset))
        }
        unsafe extern "system" fn SetRadiusX<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RadialGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, radiusx: f32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetRadiusX(::core::mem::transmute_copy(&radiusx))
        }
        unsafe extern "system" fn SetRadiusY<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RadialGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, radiusy: f32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetRadiusY(::core::mem::transmute_copy(&radiusy))
        }
        unsafe extern "system" fn GetCenter<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RadialGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D_POINT_2F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetCenter()
        }
        unsafe extern "system" fn GetGradientOriginOffset<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RadialGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D_POINT_2F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetGradientOriginOffset()
        }
        unsafe extern "system" fn GetRadiusX<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RadialGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> f32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetRadiusX()
        }
        unsafe extern "system" fn GetRadiusY<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RadialGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> f32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetRadiusY()
        }
        unsafe extern "system" fn GetGradientStopCollection<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RadialGradientBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gradientstopcollection: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetGradientStopCollection(::core::mem::transmute_copy(&gradientstopcollection))
        }
        Self {
            base: ID2D1Brush_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetCenter: SetCenter::<Identity, Impl, OFFSET>,
            SetGradientOriginOffset: SetGradientOriginOffset::<Identity, Impl, OFFSET>,
            SetRadiusX: SetRadiusX::<Identity, Impl, OFFSET>,
            SetRadiusY: SetRadiusY::<Identity, Impl, OFFSET>,
            GetCenter: GetCenter::<Identity, Impl, OFFSET>,
            GetGradientOriginOffset: GetGradientOriginOffset::<Identity, Impl, OFFSET>,
            GetRadiusX: GetRadiusX::<Identity, Impl, OFFSET>,
            GetRadiusY: GetRadiusY::<Identity, Impl, OFFSET>,
            GetGradientStopCollection: GetGradientStopCollection::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1RadialGradientBrush as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Brush as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1RectangleGeometry_Impl: Sized + ID2D1Resource_Impl + ID2D1Geometry_Impl {
    fn GetRect(&mut self, rect: *mut Common::D2D_RECT_F);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1RectangleGeometry_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RectangleGeometry_Impl, const OFFSET: isize>() -> ID2D1RectangleGeometry_Vtbl {
        unsafe extern "system" fn GetRect<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RectangleGeometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rect: *mut Common::D2D_RECT_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetRect(::core::mem::transmute_copy(&rect))
        }
        Self { base: ID2D1Geometry_Vtbl::new::<Identity, Impl, OFFSET>(), GetRect: GetRect::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1RectangleGeometry as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Geometry as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1RenderInfo_Impl: Sized {
    fn SetInputDescription(&mut self, inputindex: u32, inputdescription: &D2D1_INPUT_DESCRIPTION) -> ::windows::core::Result<()>;
    fn SetOutputBuffer(&mut self, bufferprecision: D2D1_BUFFER_PRECISION, channeldepth: D2D1_CHANNEL_DEPTH) -> ::windows::core::Result<()>;
    fn SetCached(&mut self, iscached: super::super::Foundation::BOOL);
    fn SetInstructionCountHint(&mut self, instructioncount: u32);
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1RenderInfo_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderInfo_Impl, const OFFSET: isize>() -> ID2D1RenderInfo_Vtbl {
        unsafe extern "system" fn SetInputDescription<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inputindex: u32, inputdescription: D2D1_INPUT_DESCRIPTION) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetInputDescription(::core::mem::transmute_copy(&inputindex), ::core::mem::transmute_copy(&inputdescription)).into()
        }
        unsafe extern "system" fn SetOutputBuffer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bufferprecision: D2D1_BUFFER_PRECISION, channeldepth: D2D1_CHANNEL_DEPTH) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetOutputBuffer(::core::mem::transmute_copy(&bufferprecision), ::core::mem::transmute_copy(&channeldepth)).into()
        }
        unsafe extern "system" fn SetCached<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, iscached: super::super::Foundation::BOOL) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetCached(::core::mem::transmute_copy(&iscached))
        }
        unsafe extern "system" fn SetInstructionCountHint<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderInfo_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, instructioncount: u32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetInstructionCountHint(::core::mem::transmute_copy(&instructioncount))
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            SetInputDescription: SetInputDescription::<Identity, Impl, OFFSET>,
            SetOutputBuffer: SetOutputBuffer::<Identity, Impl, OFFSET>,
            SetCached: SetCached::<Identity, Impl, OFFSET>,
            SetInstructionCountHint: SetInstructionCountHint::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1RenderInfo as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
pub trait ID2D1RenderTarget_Impl: Sized + ID2D1Resource_Impl {
    fn CreateBitmap(&mut self, size: &Common::D2D_SIZE_U, srcdata: *const ::core::ffi::c_void, pitch: u32, bitmapproperties: *const D2D1_BITMAP_PROPERTIES) -> ::windows::core::Result<ID2D1Bitmap>;
    fn CreateBitmapFromWicBitmap(&mut self, wicbitmapsource: &::core::option::Option<super::Imaging::IWICBitmapSource>, bitmapproperties: *const D2D1_BITMAP_PROPERTIES) -> ::windows::core::Result<ID2D1Bitmap>;
    fn CreateSharedBitmap(&mut self, riid: *const ::windows::core::GUID, data: *mut ::core::ffi::c_void, bitmapproperties: *const D2D1_BITMAP_PROPERTIES, bitmap: *mut ::core::option::Option<ID2D1Bitmap>) -> ::windows::core::Result<()>;
    fn CreateBitmapBrush(&mut self, bitmap: &::core::option::Option<ID2D1Bitmap>, bitmapbrushproperties: *const D2D1_BITMAP_BRUSH_PROPERTIES, brushproperties: *const D2D1_BRUSH_PROPERTIES) -> ::windows::core::Result<ID2D1BitmapBrush>;
    fn CreateSolidColorBrush(&mut self, color: *const Common::D2D1_COLOR_F, brushproperties: *const D2D1_BRUSH_PROPERTIES) -> ::windows::core::Result<ID2D1SolidColorBrush>;
    fn CreateGradientStopCollection(&mut self, gradientstops: *const D2D1_GRADIENT_STOP, gradientstopscount: u32, colorinterpolationgamma: D2D1_GAMMA, extendmode: D2D1_EXTEND_MODE) -> ::windows::core::Result<ID2D1GradientStopCollection>;
    fn CreateLinearGradientBrush(&mut self, lineargradientbrushproperties: *const D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES, brushproperties: *const D2D1_BRUSH_PROPERTIES, gradientstopcollection: &::core::option::Option<ID2D1GradientStopCollection>) -> ::windows::core::Result<ID2D1LinearGradientBrush>;
    fn CreateRadialGradientBrush(&mut self, radialgradientbrushproperties: *const D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES, brushproperties: *const D2D1_BRUSH_PROPERTIES, gradientstopcollection: &::core::option::Option<ID2D1GradientStopCollection>) -> ::windows::core::Result<ID2D1RadialGradientBrush>;
    fn CreateCompatibleRenderTarget(&mut self, desiredsize: *const Common::D2D_SIZE_F, desiredpixelsize: *const Common::D2D_SIZE_U, desiredformat: *const Common::D2D1_PIXEL_FORMAT, options: D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS) -> ::windows::core::Result<ID2D1BitmapRenderTarget>;
    fn CreateLayer(&mut self, size: *const Common::D2D_SIZE_F) -> ::windows::core::Result<ID2D1Layer>;
    fn CreateMesh(&mut self) -> ::windows::core::Result<ID2D1Mesh>;
    fn DrawLine(&mut self, point0: &Common::D2D_POINT_2F, point1: &Common::D2D_POINT_2F, brush: &::core::option::Option<ID2D1Brush>, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>);
    fn DrawRectangle(&mut self, rect: *const Common::D2D_RECT_F, brush: &::core::option::Option<ID2D1Brush>, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>);
    fn FillRectangle(&mut self, rect: *const Common::D2D_RECT_F, brush: &::core::option::Option<ID2D1Brush>);
    fn DrawRoundedRectangle(&mut self, roundedrect: *const D2D1_ROUNDED_RECT, brush: &::core::option::Option<ID2D1Brush>, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>);
    fn FillRoundedRectangle(&mut self, roundedrect: *const D2D1_ROUNDED_RECT, brush: &::core::option::Option<ID2D1Brush>);
    fn DrawEllipse(&mut self, ellipse: *const D2D1_ELLIPSE, brush: &::core::option::Option<ID2D1Brush>, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>);
    fn FillEllipse(&mut self, ellipse: *const D2D1_ELLIPSE, brush: &::core::option::Option<ID2D1Brush>);
    fn DrawGeometry(&mut self, geometry: &::core::option::Option<ID2D1Geometry>, brush: &::core::option::Option<ID2D1Brush>, strokewidth: f32, strokestyle: &::core::option::Option<ID2D1StrokeStyle>);
    fn FillGeometry(&mut self, geometry: &::core::option::Option<ID2D1Geometry>, brush: &::core::option::Option<ID2D1Brush>, opacitybrush: &::core::option::Option<ID2D1Brush>);
    fn FillMesh(&mut self, mesh: &::core::option::Option<ID2D1Mesh>, brush: &::core::option::Option<ID2D1Brush>);
    fn FillOpacityMask(&mut self, opacitymask: &::core::option::Option<ID2D1Bitmap>, brush: &::core::option::Option<ID2D1Brush>, content: D2D1_OPACITY_MASK_CONTENT, destinationrectangle: *const Common::D2D_RECT_F, sourcerectangle: *const Common::D2D_RECT_F);
    fn DrawBitmap(&mut self, bitmap: &::core::option::Option<ID2D1Bitmap>, destinationrectangle: *const Common::D2D_RECT_F, opacity: f32, interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE, sourcerectangle: *const Common::D2D_RECT_F);
    fn DrawText(&mut self, string: super::super::Foundation::PWSTR, stringlength: u32, textformat: &::core::option::Option<super::DirectWrite::IDWriteTextFormat>, layoutrect: *const Common::D2D_RECT_F, defaultfillbrush: &::core::option::Option<ID2D1Brush>, options: D2D1_DRAW_TEXT_OPTIONS, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE);
    fn DrawTextLayout(&mut self, origin: &Common::D2D_POINT_2F, textlayout: &::core::option::Option<super::DirectWrite::IDWriteTextLayout>, defaultfillbrush: &::core::option::Option<ID2D1Brush>, options: D2D1_DRAW_TEXT_OPTIONS);
    fn DrawGlyphRun(&mut self, baselineorigin: &Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, foregroundbrush: &::core::option::Option<ID2D1Brush>, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE);
    fn SetTransform(&mut self, transform: *const super::super::super::Foundation::Numerics::Matrix3x2);
    fn GetTransform(&mut self, transform: *mut super::super::super::Foundation::Numerics::Matrix3x2);
    fn SetAntialiasMode(&mut self, antialiasmode: D2D1_ANTIALIAS_MODE);
    fn GetAntialiasMode(&mut self) -> D2D1_ANTIALIAS_MODE;
    fn SetTextAntialiasMode(&mut self, textantialiasmode: D2D1_TEXT_ANTIALIAS_MODE);
    fn GetTextAntialiasMode(&mut self) -> D2D1_TEXT_ANTIALIAS_MODE;
    fn SetTextRenderingParams(&mut self, textrenderingparams: &::core::option::Option<super::DirectWrite::IDWriteRenderingParams>);
    fn GetTextRenderingParams(&mut self, textrenderingparams: *mut ::core::option::Option<super::DirectWrite::IDWriteRenderingParams>);
    fn SetTags(&mut self, tag1: u64, tag2: u64);
    fn GetTags(&mut self, tag1: *mut u64, tag2: *mut u64);
    fn PushLayer(&mut self, layerparameters: *const D2D1_LAYER_PARAMETERS, layer: &::core::option::Option<ID2D1Layer>);
    fn PopLayer(&mut self);
    fn Flush(&mut self, tag1: *mut u64, tag2: *mut u64) -> ::windows::core::Result<()>;
    fn SaveDrawingState(&mut self, drawingstateblock: &::core::option::Option<ID2D1DrawingStateBlock>);
    fn RestoreDrawingState(&mut self, drawingstateblock: &::core::option::Option<ID2D1DrawingStateBlock>);
    fn PushAxisAlignedClip(&mut self, cliprect: *const Common::D2D_RECT_F, antialiasmode: D2D1_ANTIALIAS_MODE);
    fn PopAxisAlignedClip(&mut self);
    fn Clear(&mut self, clearcolor: *const Common::D2D1_COLOR_F);
    fn BeginDraw(&mut self);
    fn EndDraw(&mut self, tag1: *mut u64, tag2: *mut u64) -> ::windows::core::Result<()>;
    fn GetPixelFormat(&mut self) -> Common::D2D1_PIXEL_FORMAT;
    fn SetDpi(&mut self, dpix: f32, dpiy: f32);
    fn GetDpi(&mut self, dpix: *mut f32, dpiy: *mut f32);
    fn GetSize(&mut self) -> Common::D2D_SIZE_F;
    fn GetPixelSize(&mut self) -> Common::D2D_SIZE_U;
    fn GetMaximumBitmapSize(&mut self) -> u32;
    fn IsSupported(&mut self, rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES) -> super::super::Foundation::BOOL;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_Graphics_DirectWrite", feature = "Win32_Graphics_Dxgi_Common", feature = "Win32_Graphics_Imaging"))]
impl ID2D1RenderTarget_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>() -> ID2D1RenderTarget_Vtbl {
        unsafe extern "system" fn CreateBitmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, size: Common::D2D_SIZE_U, srcdata: *const ::core::ffi::c_void, pitch: u32, bitmapproperties: *const D2D1_BITMAP_PROPERTIES, bitmap: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateBitmap(::core::mem::transmute_copy(&size), ::core::mem::transmute_copy(&srcdata), ::core::mem::transmute_copy(&pitch), ::core::mem::transmute_copy(&bitmapproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *bitmap = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateBitmapFromWicBitmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, wicbitmapsource: ::windows::core::RawPtr, bitmapproperties: *const D2D1_BITMAP_PROPERTIES, bitmap: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateBitmapFromWicBitmap(::core::mem::transmute(&wicbitmapsource), ::core::mem::transmute_copy(&bitmapproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *bitmap = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateSharedBitmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, riid: *const ::windows::core::GUID, data: *mut ::core::ffi::c_void, bitmapproperties: *const D2D1_BITMAP_PROPERTIES, bitmap: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).CreateSharedBitmap(::core::mem::transmute_copy(&riid), ::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&bitmapproperties), ::core::mem::transmute_copy(&bitmap)).into()
        }
        unsafe extern "system" fn CreateBitmapBrush<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bitmap: ::windows::core::RawPtr, bitmapbrushproperties: *const D2D1_BITMAP_BRUSH_PROPERTIES, brushproperties: *const D2D1_BRUSH_PROPERTIES, bitmapbrush: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateBitmapBrush(::core::mem::transmute(&bitmap), ::core::mem::transmute_copy(&bitmapbrushproperties), ::core::mem::transmute_copy(&brushproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *bitmapbrush = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateSolidColorBrush<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, color: *const Common::D2D1_COLOR_F, brushproperties: *const D2D1_BRUSH_PROPERTIES, solidcolorbrush: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateSolidColorBrush(::core::mem::transmute_copy(&color), ::core::mem::transmute_copy(&brushproperties)) {
                ::core::result::Result::Ok(ok__) => {
                    *solidcolorbrush = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateGradientStopCollection<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, gradientstops: *const D2D1_GRADIENT_STOP, gradientstopscount: u32, colorinterpolationgamma: D2D1_GAMMA, extendmode: D2D1_EXTEND_MODE, gradientstopcollection: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateGradientStopCollection(::core::mem::transmute_copy(&gradientstops), ::core::mem::transmute_copy(&gradientstopscount), ::core::mem::transmute_copy(&colorinterpolationgamma), ::core::mem::transmute_copy(&extendmode)) {
                ::core::result::Result::Ok(ok__) => {
                    *gradientstopcollection = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateLinearGradientBrush<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, lineargradientbrushproperties: *const D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES, brushproperties: *const D2D1_BRUSH_PROPERTIES, gradientstopcollection: ::windows::core::RawPtr, lineargradientbrush: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateLinearGradientBrush(::core::mem::transmute_copy(&lineargradientbrushproperties), ::core::mem::transmute_copy(&brushproperties), ::core::mem::transmute(&gradientstopcollection)) {
                ::core::result::Result::Ok(ok__) => {
                    *lineargradientbrush = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateRadialGradientBrush<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, radialgradientbrushproperties: *const D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES, brushproperties: *const D2D1_BRUSH_PROPERTIES, gradientstopcollection: ::windows::core::RawPtr, radialgradientbrush: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateRadialGradientBrush(::core::mem::transmute_copy(&radialgradientbrushproperties), ::core::mem::transmute_copy(&brushproperties), ::core::mem::transmute(&gradientstopcollection)) {
                ::core::result::Result::Ok(ok__) => {
                    *radialgradientbrush = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateCompatibleRenderTarget<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, desiredsize: *const Common::D2D_SIZE_F, desiredpixelsize: *const Common::D2D_SIZE_U, desiredformat: *const Common::D2D1_PIXEL_FORMAT, options: D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS, bitmaprendertarget: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateCompatibleRenderTarget(::core::mem::transmute_copy(&desiredsize), ::core::mem::transmute_copy(&desiredpixelsize), ::core::mem::transmute_copy(&desiredformat), ::core::mem::transmute_copy(&options)) {
                ::core::result::Result::Ok(ok__) => {
                    *bitmaprendertarget = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateLayer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, size: *const Common::D2D_SIZE_F, layer: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateLayer(::core::mem::transmute_copy(&size)) {
                ::core::result::Result::Ok(ok__) => {
                    *layer = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateMesh<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, mesh: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateMesh() {
                ::core::result::Result::Ok(ok__) => {
                    *mesh = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn DrawLine<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, point0: Common::D2D_POINT_2F, point1: Common::D2D_POINT_2F, brush: ::windows::core::RawPtr, strokewidth: f32, strokestyle: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawLine(::core::mem::transmute_copy(&point0), ::core::mem::transmute_copy(&point1), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle))
        }
        unsafe extern "system" fn DrawRectangle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rect: *const Common::D2D_RECT_F, brush: ::windows::core::RawPtr, strokewidth: f32, strokestyle: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawRectangle(::core::mem::transmute_copy(&rect), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle))
        }
        unsafe extern "system" fn FillRectangle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rect: *const Common::D2D_RECT_F, brush: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillRectangle(::core::mem::transmute_copy(&rect), ::core::mem::transmute(&brush))
        }
        unsafe extern "system" fn DrawRoundedRectangle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, roundedrect: *const D2D1_ROUNDED_RECT, brush: ::windows::core::RawPtr, strokewidth: f32, strokestyle: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawRoundedRectangle(::core::mem::transmute_copy(&roundedrect), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle))
        }
        unsafe extern "system" fn FillRoundedRectangle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, roundedrect: *const D2D1_ROUNDED_RECT, brush: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillRoundedRectangle(::core::mem::transmute_copy(&roundedrect), ::core::mem::transmute(&brush))
        }
        unsafe extern "system" fn DrawEllipse<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, ellipse: *const D2D1_ELLIPSE, brush: ::windows::core::RawPtr, strokewidth: f32, strokestyle: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawEllipse(::core::mem::transmute_copy(&ellipse), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle))
        }
        unsafe extern "system" fn FillEllipse<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, ellipse: *const D2D1_ELLIPSE, brush: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillEllipse(::core::mem::transmute_copy(&ellipse), ::core::mem::transmute(&brush))
        }
        unsafe extern "system" fn DrawGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, geometry: ::windows::core::RawPtr, brush: ::windows::core::RawPtr, strokewidth: f32, strokestyle: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGeometry(::core::mem::transmute(&geometry), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute(&strokestyle))
        }
        unsafe extern "system" fn FillGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, geometry: ::windows::core::RawPtr, brush: ::windows::core::RawPtr, opacitybrush: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillGeometry(::core::mem::transmute(&geometry), ::core::mem::transmute(&brush), ::core::mem::transmute(&opacitybrush))
        }
        unsafe extern "system" fn FillMesh<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, mesh: ::windows::core::RawPtr, brush: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillMesh(::core::mem::transmute(&mesh), ::core::mem::transmute(&brush))
        }
        unsafe extern "system" fn FillOpacityMask<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, opacitymask: ::windows::core::RawPtr, brush: ::windows::core::RawPtr, content: D2D1_OPACITY_MASK_CONTENT, destinationrectangle: *const Common::D2D_RECT_F, sourcerectangle: *const Common::D2D_RECT_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).FillOpacityMask(::core::mem::transmute(&opacitymask), ::core::mem::transmute(&brush), ::core::mem::transmute_copy(&content), ::core::mem::transmute_copy(&destinationrectangle), ::core::mem::transmute_copy(&sourcerectangle))
        }
        unsafe extern "system" fn DrawBitmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, bitmap: ::windows::core::RawPtr, destinationrectangle: *const Common::D2D_RECT_F, opacity: f32, interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE, sourcerectangle: *const Common::D2D_RECT_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawBitmap(::core::mem::transmute(&bitmap), ::core::mem::transmute_copy(&destinationrectangle), ::core::mem::transmute_copy(&opacity), ::core::mem::transmute_copy(&interpolationmode), ::core::mem::transmute_copy(&sourcerectangle))
        }
        unsafe extern "system" fn DrawText<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, string: super::super::Foundation::PWSTR, stringlength: u32, textformat: ::windows::core::RawPtr, layoutrect: *const Common::D2D_RECT_F, defaultfillbrush: ::windows::core::RawPtr, options: D2D1_DRAW_TEXT_OPTIONS, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawText(::core::mem::transmute_copy(&string), ::core::mem::transmute_copy(&stringlength), ::core::mem::transmute(&textformat), ::core::mem::transmute_copy(&layoutrect), ::core::mem::transmute(&defaultfillbrush), ::core::mem::transmute_copy(&options), ::core::mem::transmute_copy(&measuringmode))
        }
        unsafe extern "system" fn DrawTextLayout<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, origin: Common::D2D_POINT_2F, textlayout: ::windows::core::RawPtr, defaultfillbrush: ::windows::core::RawPtr, options: D2D1_DRAW_TEXT_OPTIONS) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawTextLayout(::core::mem::transmute_copy(&origin), ::core::mem::transmute(&textlayout), ::core::mem::transmute(&defaultfillbrush), ::core::mem::transmute_copy(&options))
        }
        unsafe extern "system" fn DrawGlyphRun<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, baselineorigin: Common::D2D_POINT_2F, glyphrun: *const super::DirectWrite::DWRITE_GLYPH_RUN, foregroundbrush: ::windows::core::RawPtr, measuringmode: super::DirectWrite::DWRITE_MEASURING_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).DrawGlyphRun(::core::mem::transmute_copy(&baselineorigin), ::core::mem::transmute_copy(&glyphrun), ::core::mem::transmute(&foregroundbrush), ::core::mem::transmute_copy(&measuringmode))
        }
        unsafe extern "system" fn SetTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, transform: *const super::super::super::Foundation::Numerics::Matrix3x2) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTransform(::core::mem::transmute_copy(&transform))
        }
        unsafe extern "system" fn GetTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, transform: *mut super::super::super::Foundation::Numerics::Matrix3x2) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTransform(::core::mem::transmute_copy(&transform))
        }
        unsafe extern "system" fn SetAntialiasMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, antialiasmode: D2D1_ANTIALIAS_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetAntialiasMode(::core::mem::transmute_copy(&antialiasmode))
        }
        unsafe extern "system" fn GetAntialiasMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_ANTIALIAS_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetAntialiasMode()
        }
        unsafe extern "system" fn SetTextAntialiasMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, textantialiasmode: D2D1_TEXT_ANTIALIAS_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTextAntialiasMode(::core::mem::transmute_copy(&textantialiasmode))
        }
        unsafe extern "system" fn GetTextAntialiasMode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_TEXT_ANTIALIAS_MODE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTextAntialiasMode()
        }
        unsafe extern "system" fn SetTextRenderingParams<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, textrenderingparams: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTextRenderingParams(::core::mem::transmute(&textrenderingparams))
        }
        unsafe extern "system" fn GetTextRenderingParams<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, textrenderingparams: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTextRenderingParams(::core::mem::transmute_copy(&textrenderingparams))
        }
        unsafe extern "system" fn SetTags<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, tag1: u64, tag2: u64) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTags(::core::mem::transmute_copy(&tag1), ::core::mem::transmute_copy(&tag2))
        }
        unsafe extern "system" fn GetTags<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, tag1: *mut u64, tag2: *mut u64) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTags(::core::mem::transmute_copy(&tag1), ::core::mem::transmute_copy(&tag2))
        }
        unsafe extern "system" fn PushLayer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, layerparameters: *const D2D1_LAYER_PARAMETERS, layer: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).PushLayer(::core::mem::transmute_copy(&layerparameters), ::core::mem::transmute(&layer))
        }
        unsafe extern "system" fn PopLayer<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).PopLayer()
        }
        unsafe extern "system" fn Flush<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, tag1: *mut u64, tag2: *mut u64) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Flush(::core::mem::transmute_copy(&tag1), ::core::mem::transmute_copy(&tag2)).into()
        }
        unsafe extern "system" fn SaveDrawingState<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, drawingstateblock: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SaveDrawingState(::core::mem::transmute(&drawingstateblock))
        }
        unsafe extern "system" fn RestoreDrawingState<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, drawingstateblock: ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RestoreDrawingState(::core::mem::transmute(&drawingstateblock))
        }
        unsafe extern "system" fn PushAxisAlignedClip<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, cliprect: *const Common::D2D_RECT_F, antialiasmode: D2D1_ANTIALIAS_MODE) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).PushAxisAlignedClip(::core::mem::transmute_copy(&cliprect), ::core::mem::transmute_copy(&antialiasmode))
        }
        unsafe extern "system" fn PopAxisAlignedClip<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).PopAxisAlignedClip()
        }
        unsafe extern "system" fn Clear<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, clearcolor: *const Common::D2D1_COLOR_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Clear(::core::mem::transmute_copy(&clearcolor))
        }
        unsafe extern "system" fn BeginDraw<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).BeginDraw()
        }
        unsafe extern "system" fn EndDraw<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, tag1: *mut u64, tag2: *mut u64) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).EndDraw(::core::mem::transmute_copy(&tag1), ::core::mem::transmute_copy(&tag2)).into()
        }
        unsafe extern "system" fn GetPixelFormat<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D1_PIXEL_FORMAT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetPixelFormat()
        }
        unsafe extern "system" fn SetDpi<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dpix: f32, dpiy: f32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetDpi(::core::mem::transmute_copy(&dpix), ::core::mem::transmute_copy(&dpiy))
        }
        unsafe extern "system" fn GetDpi<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dpix: *mut f32, dpiy: *mut f32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDpi(::core::mem::transmute_copy(&dpix), ::core::mem::transmute_copy(&dpiy))
        }
        unsafe extern "system" fn GetSize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D_SIZE_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetSize()
        }
        unsafe extern "system" fn GetPixelSize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D_SIZE_U) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetPixelSize()
        }
        unsafe extern "system" fn GetMaximumBitmapSize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetMaximumBitmapSize()
        }
        unsafe extern "system" fn IsSupported<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RenderTarget_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES) -> super::super::Foundation::BOOL {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).IsSupported(::core::mem::transmute_copy(&rendertargetproperties))
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            CreateBitmap: CreateBitmap::<Identity, Impl, OFFSET>,
            CreateBitmapFromWicBitmap: CreateBitmapFromWicBitmap::<Identity, Impl, OFFSET>,
            CreateSharedBitmap: CreateSharedBitmap::<Identity, Impl, OFFSET>,
            CreateBitmapBrush: CreateBitmapBrush::<Identity, Impl, OFFSET>,
            CreateSolidColorBrush: CreateSolidColorBrush::<Identity, Impl, OFFSET>,
            CreateGradientStopCollection: CreateGradientStopCollection::<Identity, Impl, OFFSET>,
            CreateLinearGradientBrush: CreateLinearGradientBrush::<Identity, Impl, OFFSET>,
            CreateRadialGradientBrush: CreateRadialGradientBrush::<Identity, Impl, OFFSET>,
            CreateCompatibleRenderTarget: CreateCompatibleRenderTarget::<Identity, Impl, OFFSET>,
            CreateLayer: CreateLayer::<Identity, Impl, OFFSET>,
            CreateMesh: CreateMesh::<Identity, Impl, OFFSET>,
            DrawLine: DrawLine::<Identity, Impl, OFFSET>,
            DrawRectangle: DrawRectangle::<Identity, Impl, OFFSET>,
            FillRectangle: FillRectangle::<Identity, Impl, OFFSET>,
            DrawRoundedRectangle: DrawRoundedRectangle::<Identity, Impl, OFFSET>,
            FillRoundedRectangle: FillRoundedRectangle::<Identity, Impl, OFFSET>,
            DrawEllipse: DrawEllipse::<Identity, Impl, OFFSET>,
            FillEllipse: FillEllipse::<Identity, Impl, OFFSET>,
            DrawGeometry: DrawGeometry::<Identity, Impl, OFFSET>,
            FillGeometry: FillGeometry::<Identity, Impl, OFFSET>,
            FillMesh: FillMesh::<Identity, Impl, OFFSET>,
            FillOpacityMask: FillOpacityMask::<Identity, Impl, OFFSET>,
            DrawBitmap: DrawBitmap::<Identity, Impl, OFFSET>,
            DrawText: DrawText::<Identity, Impl, OFFSET>,
            DrawTextLayout: DrawTextLayout::<Identity, Impl, OFFSET>,
            DrawGlyphRun: DrawGlyphRun::<Identity, Impl, OFFSET>,
            SetTransform: SetTransform::<Identity, Impl, OFFSET>,
            GetTransform: GetTransform::<Identity, Impl, OFFSET>,
            SetAntialiasMode: SetAntialiasMode::<Identity, Impl, OFFSET>,
            GetAntialiasMode: GetAntialiasMode::<Identity, Impl, OFFSET>,
            SetTextAntialiasMode: SetTextAntialiasMode::<Identity, Impl, OFFSET>,
            GetTextAntialiasMode: GetTextAntialiasMode::<Identity, Impl, OFFSET>,
            SetTextRenderingParams: SetTextRenderingParams::<Identity, Impl, OFFSET>,
            GetTextRenderingParams: GetTextRenderingParams::<Identity, Impl, OFFSET>,
            SetTags: SetTags::<Identity, Impl, OFFSET>,
            GetTags: GetTags::<Identity, Impl, OFFSET>,
            PushLayer: PushLayer::<Identity, Impl, OFFSET>,
            PopLayer: PopLayer::<Identity, Impl, OFFSET>,
            Flush: Flush::<Identity, Impl, OFFSET>,
            SaveDrawingState: SaveDrawingState::<Identity, Impl, OFFSET>,
            RestoreDrawingState: RestoreDrawingState::<Identity, Impl, OFFSET>,
            PushAxisAlignedClip: PushAxisAlignedClip::<Identity, Impl, OFFSET>,
            PopAxisAlignedClip: PopAxisAlignedClip::<Identity, Impl, OFFSET>,
            Clear: Clear::<Identity, Impl, OFFSET>,
            BeginDraw: BeginDraw::<Identity, Impl, OFFSET>,
            EndDraw: EndDraw::<Identity, Impl, OFFSET>,
            GetPixelFormat: GetPixelFormat::<Identity, Impl, OFFSET>,
            SetDpi: SetDpi::<Identity, Impl, OFFSET>,
            GetDpi: GetDpi::<Identity, Impl, OFFSET>,
            GetSize: GetSize::<Identity, Impl, OFFSET>,
            GetPixelSize: GetPixelSize::<Identity, Impl, OFFSET>,
            GetMaximumBitmapSize: GetMaximumBitmapSize::<Identity, Impl, OFFSET>,
            IsSupported: IsSupported::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1RenderTarget as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1Resource_Impl: Sized {
    fn GetFactory(&mut self, factory: *mut ::core::option::Option<ID2D1Factory>);
}
impl ID2D1Resource_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Resource_Impl, const OFFSET: isize>() -> ID2D1Resource_Vtbl {
        unsafe extern "system" fn GetFactory<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Resource_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, factory: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetFactory(::core::mem::transmute_copy(&factory))
        }
        Self { base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(), GetFactory: GetFactory::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1ResourceTexture_Impl: Sized {
    fn Update(&mut self, minimumextents: *const u32, maximimumextents: *const u32, strides: *const u32, dimensions: u32, data: *const u8, datacount: u32) -> ::windows::core::Result<()>;
}
impl ID2D1ResourceTexture_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ResourceTexture_Impl, const OFFSET: isize>() -> ID2D1ResourceTexture_Vtbl {
        unsafe extern "system" fn Update<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1ResourceTexture_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, minimumextents: *const u32, maximimumextents: *const u32, strides: *const u32, dimensions: u32, data: *const u8, datacount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Update(::core::mem::transmute_copy(&minimumextents), ::core::mem::transmute_copy(&maximimumextents), ::core::mem::transmute_copy(&strides), ::core::mem::transmute_copy(&dimensions), ::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&datacount)).into()
        }
        Self { base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(), Update: Update::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1ResourceTexture as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1RoundedRectangleGeometry_Impl: Sized + ID2D1Resource_Impl + ID2D1Geometry_Impl {
    fn GetRoundedRect(&mut self, roundedrect: *mut D2D1_ROUNDED_RECT);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1RoundedRectangleGeometry_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RoundedRectangleGeometry_Impl, const OFFSET: isize>() -> ID2D1RoundedRectangleGeometry_Vtbl {
        unsafe extern "system" fn GetRoundedRect<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1RoundedRectangleGeometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, roundedrect: *mut D2D1_ROUNDED_RECT) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetRoundedRect(::core::mem::transmute_copy(&roundedrect))
        }
        Self { base: ID2D1Geometry_Vtbl::new::<Identity, Impl, OFFSET>(), GetRoundedRect: GetRoundedRect::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1RoundedRectangleGeometry as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Geometry as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1SolidColorBrush_Impl: Sized + ID2D1Resource_Impl + ID2D1Brush_Impl {
    fn SetColor(&mut self, color: *const Common::D2D1_COLOR_F);
    fn GetColor(&mut self) -> Common::D2D1_COLOR_F;
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1SolidColorBrush_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SolidColorBrush_Impl, const OFFSET: isize>() -> ID2D1SolidColorBrush_Vtbl {
        unsafe extern "system" fn SetColor<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SolidColorBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, color: *const Common::D2D1_COLOR_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetColor(::core::mem::transmute_copy(&color))
        }
        unsafe extern "system" fn GetColor<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SolidColorBrush_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D1_COLOR_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetColor()
        }
        Self {
            base: ID2D1Brush_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetColor: SetColor::<Identity, Impl, OFFSET>,
            GetColor: GetColor::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SolidColorBrush as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Brush as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1SourceTransform_Impl: Sized + ID2D1TransformNode_Impl + ID2D1Transform_Impl {
    fn SetRenderInfo(&mut self, renderinfo: &::core::option::Option<ID2D1RenderInfo>) -> ::windows::core::Result<()>;
    fn Draw(&mut self, target: &::core::option::Option<ID2D1Bitmap1>, drawrect: *const super::super::Foundation::RECT, targetorigin: &Common::D2D_POINT_2U) -> ::windows::core::Result<()>;
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1SourceTransform_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SourceTransform_Impl, const OFFSET: isize>() -> ID2D1SourceTransform_Vtbl {
        unsafe extern "system" fn SetRenderInfo<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SourceTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, renderinfo: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetRenderInfo(::core::mem::transmute(&renderinfo)).into()
        }
        unsafe extern "system" fn Draw<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SourceTransform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, target: ::windows::core::RawPtr, drawrect: *const super::super::Foundation::RECT, targetorigin: Common::D2D_POINT_2U) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Draw(::core::mem::transmute(&target), ::core::mem::transmute_copy(&drawrect), ::core::mem::transmute_copy(&targetorigin)).into()
        }
        Self {
            base: ID2D1Transform_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetRenderInfo: SetRenderInfo::<Identity, Impl, OFFSET>,
            Draw: Draw::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SourceTransform as ::windows::core::Interface>::IID || iid == &<ID2D1TransformNode as ::windows::core::Interface>::IID || iid == &<ID2D1Transform as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1SpriteBatch_Impl: Sized + ID2D1Resource_Impl {
    fn AddSprites(&mut self, spritecount: u32, destinationrectangles: *const Common::D2D_RECT_F, sourcerectangles: *const Common::D2D_RECT_U, colors: *const Common::D2D1_COLOR_F, transforms: *const super::super::super::Foundation::Numerics::Matrix3x2, destinationrectanglesstride: u32, sourcerectanglesstride: u32, colorsstride: u32, transformsstride: u32) -> ::windows::core::Result<()>;
    fn SetSprites(&mut self, startindex: u32, spritecount: u32, destinationrectangles: *const Common::D2D_RECT_F, sourcerectangles: *const Common::D2D_RECT_U, colors: *const Common::D2D1_COLOR_F, transforms: *const super::super::super::Foundation::Numerics::Matrix3x2, destinationrectanglesstride: u32, sourcerectanglesstride: u32, colorsstride: u32, transformsstride: u32) -> ::windows::core::Result<()>;
    fn GetSprites(&mut self, startindex: u32, spritecount: u32, destinationrectangles: *mut Common::D2D_RECT_F, sourcerectangles: *mut Common::D2D_RECT_U, colors: *mut Common::D2D1_COLOR_F, transforms: *mut super::super::super::Foundation::Numerics::Matrix3x2) -> ::windows::core::Result<()>;
    fn GetSpriteCount(&mut self) -> u32;
    fn Clear(&mut self);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1SpriteBatch_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SpriteBatch_Impl, const OFFSET: isize>() -> ID2D1SpriteBatch_Vtbl {
        unsafe extern "system" fn AddSprites<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SpriteBatch_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, spritecount: u32, destinationrectangles: *const Common::D2D_RECT_F, sourcerectangles: *const Common::D2D_RECT_U, colors: *const Common::D2D1_COLOR_F, transforms: *const super::super::super::Foundation::Numerics::Matrix3x2, destinationrectanglesstride: u32, sourcerectanglesstride: u32, colorsstride: u32, transformsstride: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AddSprites(::core::mem::transmute_copy(&spritecount), ::core::mem::transmute_copy(&destinationrectangles), ::core::mem::transmute_copy(&sourcerectangles), ::core::mem::transmute_copy(&colors), ::core::mem::transmute_copy(&transforms), ::core::mem::transmute_copy(&destinationrectanglesstride), ::core::mem::transmute_copy(&sourcerectanglesstride), ::core::mem::transmute_copy(&colorsstride), ::core::mem::transmute_copy(&transformsstride)).into()
        }
        unsafe extern "system" fn SetSprites<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SpriteBatch_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, startindex: u32, spritecount: u32, destinationrectangles: *const Common::D2D_RECT_F, sourcerectangles: *const Common::D2D_RECT_U, colors: *const Common::D2D1_COLOR_F, transforms: *const super::super::super::Foundation::Numerics::Matrix3x2, destinationrectanglesstride: u32, sourcerectanglesstride: u32, colorsstride: u32, transformsstride: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this)
                .SetSprites(::core::mem::transmute_copy(&startindex), ::core::mem::transmute_copy(&spritecount), ::core::mem::transmute_copy(&destinationrectangles), ::core::mem::transmute_copy(&sourcerectangles), ::core::mem::transmute_copy(&colors), ::core::mem::transmute_copy(&transforms), ::core::mem::transmute_copy(&destinationrectanglesstride), ::core::mem::transmute_copy(&sourcerectanglesstride), ::core::mem::transmute_copy(&colorsstride), ::core::mem::transmute_copy(&transformsstride))
                .into()
        }
        unsafe extern "system" fn GetSprites<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SpriteBatch_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, startindex: u32, spritecount: u32, destinationrectangles: *mut Common::D2D_RECT_F, sourcerectangles: *mut Common::D2D_RECT_U, colors: *mut Common::D2D1_COLOR_F, transforms: *mut super::super::super::Foundation::Numerics::Matrix3x2) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSprites(::core::mem::transmute_copy(&startindex), ::core::mem::transmute_copy(&spritecount), ::core::mem::transmute_copy(&destinationrectangles), ::core::mem::transmute_copy(&sourcerectangles), ::core::mem::transmute_copy(&colors), ::core::mem::transmute_copy(&transforms)).into()
        }
        unsafe extern "system" fn GetSpriteCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SpriteBatch_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSpriteCount()
        }
        unsafe extern "system" fn Clear<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SpriteBatch_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Clear()
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            AddSprites: AddSprites::<Identity, Impl, OFFSET>,
            SetSprites: SetSprites::<Identity, Impl, OFFSET>,
            GetSprites: GetSprites::<Identity, Impl, OFFSET>,
            GetSpriteCount: GetSpriteCount::<Identity, Impl, OFFSET>,
            Clear: Clear::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SpriteBatch as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1StrokeStyle_Impl: Sized + ID2D1Resource_Impl {
    fn GetStartCap(&mut self) -> D2D1_CAP_STYLE;
    fn GetEndCap(&mut self) -> D2D1_CAP_STYLE;
    fn GetDashCap(&mut self) -> D2D1_CAP_STYLE;
    fn GetMiterLimit(&mut self) -> f32;
    fn GetLineJoin(&mut self) -> D2D1_LINE_JOIN;
    fn GetDashOffset(&mut self) -> f32;
    fn GetDashStyle(&mut self) -> D2D1_DASH_STYLE;
    fn GetDashesCount(&mut self) -> u32;
    fn GetDashes(&mut self, dashes: *mut f32, dashescount: u32);
}
impl ID2D1StrokeStyle_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle_Impl, const OFFSET: isize>() -> ID2D1StrokeStyle_Vtbl {
        unsafe extern "system" fn GetStartCap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_CAP_STYLE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetStartCap()
        }
        unsafe extern "system" fn GetEndCap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_CAP_STYLE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetEndCap()
        }
        unsafe extern "system" fn GetDashCap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_CAP_STYLE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDashCap()
        }
        unsafe extern "system" fn GetMiterLimit<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> f32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetMiterLimit()
        }
        unsafe extern "system" fn GetLineJoin<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_LINE_JOIN {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetLineJoin()
        }
        unsafe extern "system" fn GetDashOffset<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> f32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDashOffset()
        }
        unsafe extern "system" fn GetDashStyle<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_DASH_STYLE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDashStyle()
        }
        unsafe extern "system" fn GetDashesCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDashesCount()
        }
        unsafe extern "system" fn GetDashes<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dashes: *mut f32, dashescount: u32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDashes(::core::mem::transmute_copy(&dashes), ::core::mem::transmute_copy(&dashescount))
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetStartCap: GetStartCap::<Identity, Impl, OFFSET>,
            GetEndCap: GetEndCap::<Identity, Impl, OFFSET>,
            GetDashCap: GetDashCap::<Identity, Impl, OFFSET>,
            GetMiterLimit: GetMiterLimit::<Identity, Impl, OFFSET>,
            GetLineJoin: GetLineJoin::<Identity, Impl, OFFSET>,
            GetDashOffset: GetDashOffset::<Identity, Impl, OFFSET>,
            GetDashStyle: GetDashStyle::<Identity, Impl, OFFSET>,
            GetDashesCount: GetDashesCount::<Identity, Impl, OFFSET>,
            GetDashes: GetDashes::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1StrokeStyle as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1StrokeStyle1_Impl: Sized + ID2D1Resource_Impl + ID2D1StrokeStyle_Impl {
    fn GetStrokeTransformType(&mut self) -> D2D1_STROKE_TRANSFORM_TYPE;
}
impl ID2D1StrokeStyle1_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle1_Impl, const OFFSET: isize>() -> ID2D1StrokeStyle1_Vtbl {
        unsafe extern "system" fn GetStrokeTransformType<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1StrokeStyle1_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_STROKE_TRANSFORM_TYPE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetStrokeTransformType()
        }
        Self { base: ID2D1StrokeStyle_Vtbl::new::<Identity, Impl, OFFSET>(), GetStrokeTransformType: GetStrokeTransformType::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1StrokeStyle1 as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1StrokeStyle as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1SvgAttribute_Impl: Sized + ID2D1Resource_Impl {
    fn GetElement(&mut self, element: *mut ::core::option::Option<ID2D1SvgElement>);
    fn Clone(&mut self) -> ::windows::core::Result<ID2D1SvgAttribute>;
}
impl ID2D1SvgAttribute_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgAttribute_Impl, const OFFSET: isize>() -> ID2D1SvgAttribute_Vtbl {
        unsafe extern "system" fn GetElement<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgAttribute_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, element: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetElement(::core::mem::transmute_copy(&element))
        }
        unsafe extern "system" fn Clone<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgAttribute_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, attribute: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).Clone() {
                ::core::result::Result::Ok(ok__) => {
                    *attribute = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetElement: GetElement::<Identity, Impl, OFFSET>,
            Clone: Clone::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SvgAttribute as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_System_Com"))]
pub trait ID2D1SvgDocument_Impl: Sized + ID2D1Resource_Impl {
    fn SetViewportSize(&mut self, viewportsize: &Common::D2D_SIZE_F) -> ::windows::core::Result<()>;
    fn GetViewportSize(&mut self) -> Common::D2D_SIZE_F;
    fn SetRoot(&mut self, root: &::core::option::Option<ID2D1SvgElement>) -> ::windows::core::Result<()>;
    fn GetRoot(&mut self, root: *mut ::core::option::Option<ID2D1SvgElement>);
    fn FindElementById(&mut self, id: super::super::Foundation::PWSTR) -> ::windows::core::Result<ID2D1SvgElement>;
    fn Serialize(&mut self, outputxmlstream: &::core::option::Option<super::super::System::Com::IStream>, subtree: &::core::option::Option<ID2D1SvgElement>) -> ::windows::core::Result<()>;
    fn Deserialize(&mut self, inputxmlstream: &::core::option::Option<super::super::System::Com::IStream>) -> ::windows::core::Result<ID2D1SvgElement>;
    fn CreatePaint(&mut self, painttype: D2D1_SVG_PAINT_TYPE, color: *const Common::D2D1_COLOR_F, id: super::super::Foundation::PWSTR) -> ::windows::core::Result<ID2D1SvgPaint>;
    fn CreateStrokeDashArray(&mut self, dashes: *const D2D1_SVG_LENGTH, dashescount: u32) -> ::windows::core::Result<ID2D1SvgStrokeDashArray>;
    fn CreatePointCollection(&mut self, points: *const Common::D2D_POINT_2F, pointscount: u32) -> ::windows::core::Result<ID2D1SvgPointCollection>;
    fn CreatePathData(&mut self, segmentdata: *const f32, segmentdatacount: u32, commands: *const D2D1_SVG_PATH_COMMAND, commandscount: u32) -> ::windows::core::Result<ID2D1SvgPathData>;
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common", feature = "Win32_System_Com"))]
impl ID2D1SvgDocument_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>() -> ID2D1SvgDocument_Vtbl {
        unsafe extern "system" fn SetViewportSize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, viewportsize: Common::D2D_SIZE_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetViewportSize(::core::mem::transmute_copy(&viewportsize)).into()
        }
        unsafe extern "system" fn GetViewportSize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, result__: *mut Common::D2D_SIZE_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            *result__ = (*this).GetViewportSize()
        }
        unsafe extern "system" fn SetRoot<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, root: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetRoot(::core::mem::transmute(&root)).into()
        }
        unsafe extern "system" fn GetRoot<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, root: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetRoot(::core::mem::transmute_copy(&root))
        }
        unsafe extern "system" fn FindElementById<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, id: super::super::Foundation::PWSTR, svgelement: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).FindElementById(::core::mem::transmute_copy(&id)) {
                ::core::result::Result::Ok(ok__) => {
                    *svgelement = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn Serialize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, outputxmlstream: ::windows::core::RawPtr, subtree: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Serialize(::core::mem::transmute(&outputxmlstream), ::core::mem::transmute(&subtree)).into()
        }
        unsafe extern "system" fn Deserialize<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inputxmlstream: ::windows::core::RawPtr, subtree: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).Deserialize(::core::mem::transmute(&inputxmlstream)) {
                ::core::result::Result::Ok(ok__) => {
                    *subtree = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreatePaint<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, painttype: D2D1_SVG_PAINT_TYPE, color: *const Common::D2D1_COLOR_F, id: super::super::Foundation::PWSTR, paint: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreatePaint(::core::mem::transmute_copy(&painttype), ::core::mem::transmute_copy(&color), ::core::mem::transmute_copy(&id)) {
                ::core::result::Result::Ok(ok__) => {
                    *paint = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreateStrokeDashArray<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dashes: *const D2D1_SVG_LENGTH, dashescount: u32, strokedasharray: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateStrokeDashArray(::core::mem::transmute_copy(&dashes), ::core::mem::transmute_copy(&dashescount)) {
                ::core::result::Result::Ok(ok__) => {
                    *strokedasharray = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreatePointCollection<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, points: *const Common::D2D_POINT_2F, pointscount: u32, pointcollection: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreatePointCollection(::core::mem::transmute_copy(&points), ::core::mem::transmute_copy(&pointscount)) {
                ::core::result::Result::Ok(ok__) => {
                    *pointcollection = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn CreatePathData<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgDocument_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, segmentdata: *const f32, segmentdatacount: u32, commands: *const D2D1_SVG_PATH_COMMAND, commandscount: u32, pathdata: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreatePathData(::core::mem::transmute_copy(&segmentdata), ::core::mem::transmute_copy(&segmentdatacount), ::core::mem::transmute_copy(&commands), ::core::mem::transmute_copy(&commandscount)) {
                ::core::result::Result::Ok(ok__) => {
                    *pathdata = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetViewportSize: SetViewportSize::<Identity, Impl, OFFSET>,
            GetViewportSize: GetViewportSize::<Identity, Impl, OFFSET>,
            SetRoot: SetRoot::<Identity, Impl, OFFSET>,
            GetRoot: GetRoot::<Identity, Impl, OFFSET>,
            FindElementById: FindElementById::<Identity, Impl, OFFSET>,
            Serialize: Serialize::<Identity, Impl, OFFSET>,
            Deserialize: Deserialize::<Identity, Impl, OFFSET>,
            CreatePaint: CreatePaint::<Identity, Impl, OFFSET>,
            CreateStrokeDashArray: CreateStrokeDashArray::<Identity, Impl, OFFSET>,
            CreatePointCollection: CreatePointCollection::<Identity, Impl, OFFSET>,
            CreatePathData: CreatePathData::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SvgDocument as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1SvgElement_Impl: Sized + ID2D1Resource_Impl {
    fn GetDocument(&mut self, document: *mut ::core::option::Option<ID2D1SvgDocument>);
    fn GetTagName(&mut self, name: super::super::Foundation::PWSTR, namecount: u32) -> ::windows::core::Result<()>;
    fn GetTagNameLength(&mut self) -> u32;
    fn IsTextContent(&mut self) -> super::super::Foundation::BOOL;
    fn GetParent(&mut self, parent: *mut ::core::option::Option<ID2D1SvgElement>);
    fn HasChildren(&mut self) -> super::super::Foundation::BOOL;
    fn GetFirstChild(&mut self, child: *mut ::core::option::Option<ID2D1SvgElement>);
    fn GetLastChild(&mut self, child: *mut ::core::option::Option<ID2D1SvgElement>);
    fn GetPreviousChild(&mut self, referencechild: &::core::option::Option<ID2D1SvgElement>) -> ::windows::core::Result<ID2D1SvgElement>;
    fn GetNextChild(&mut self, referencechild: &::core::option::Option<ID2D1SvgElement>) -> ::windows::core::Result<ID2D1SvgElement>;
    fn InsertChildBefore(&mut self, newchild: &::core::option::Option<ID2D1SvgElement>, referencechild: &::core::option::Option<ID2D1SvgElement>) -> ::windows::core::Result<()>;
    fn AppendChild(&mut self, newchild: &::core::option::Option<ID2D1SvgElement>) -> ::windows::core::Result<()>;
    fn ReplaceChild(&mut self, newchild: &::core::option::Option<ID2D1SvgElement>, oldchild: &::core::option::Option<ID2D1SvgElement>) -> ::windows::core::Result<()>;
    fn RemoveChild(&mut self, oldchild: &::core::option::Option<ID2D1SvgElement>) -> ::windows::core::Result<()>;
    fn CreateChild(&mut self, tagname: super::super::Foundation::PWSTR) -> ::windows::core::Result<ID2D1SvgElement>;
    fn IsAttributeSpecified(&mut self, name: super::super::Foundation::PWSTR, inherited: *mut super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    fn GetSpecifiedAttributeCount(&mut self) -> u32;
    fn GetSpecifiedAttributeName(&mut self, index: u32, name: super::super::Foundation::PWSTR, namecount: u32, inherited: *mut super::super::Foundation::BOOL) -> ::windows::core::Result<()>;
    fn GetSpecifiedAttributeNameLength(&mut self, index: u32, namelength: *mut u32, inherited: *mut super::super::Foundation::BOOL) -> ::windows::core::Result<()>;
    fn RemoveAttribute(&mut self, name: super::super::Foundation::PWSTR) -> ::windows::core::Result<()>;
    fn SetTextValue(&mut self, name: super::super::Foundation::PWSTR, namecount: u32) -> ::windows::core::Result<()>;
    fn GetTextValue(&mut self, name: super::super::Foundation::PWSTR, namecount: u32) -> ::windows::core::Result<()>;
    fn GetTextValueLength(&mut self) -> u32;
    fn SetAttributeValue(&mut self, name: super::super::Foundation::PWSTR, value: &::core::option::Option<ID2D1SvgAttribute>) -> ::windows::core::Result<()>;
    fn SetAttributeValue2(&mut self, name: super::super::Foundation::PWSTR, r#type: D2D1_SVG_ATTRIBUTE_POD_TYPE, value: *const ::core::ffi::c_void, valuesizeinbytes: u32) -> ::windows::core::Result<()>;
    fn SetAttributeValue3(&mut self, name: super::super::Foundation::PWSTR, r#type: D2D1_SVG_ATTRIBUTE_STRING_TYPE, value: super::super::Foundation::PWSTR) -> ::windows::core::Result<()>;
    fn GetAttributeValue(&mut self, name: super::super::Foundation::PWSTR, riid: *const ::windows::core::GUID, value: *mut *mut ::core::ffi::c_void) -> ::windows::core::Result<()>;
    fn GetAttributeValue2(&mut self, name: super::super::Foundation::PWSTR, r#type: D2D1_SVG_ATTRIBUTE_POD_TYPE, value: *mut ::core::ffi::c_void, valuesizeinbytes: u32) -> ::windows::core::Result<()>;
    fn GetAttributeValue3(&mut self, name: super::super::Foundation::PWSTR, r#type: D2D1_SVG_ATTRIBUTE_STRING_TYPE, value: super::super::Foundation::PWSTR, valuecount: u32) -> ::windows::core::Result<()>;
    fn GetAttributeValueLength(&mut self, name: super::super::Foundation::PWSTR, r#type: D2D1_SVG_ATTRIBUTE_STRING_TYPE) -> ::windows::core::Result<u32>;
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1SvgElement_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>() -> ID2D1SvgElement_Vtbl {
        unsafe extern "system" fn GetDocument<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, document: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDocument(::core::mem::transmute_copy(&document))
        }
        unsafe extern "system" fn GetTagName<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, namecount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTagName(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&namecount)).into()
        }
        unsafe extern "system" fn GetTagNameLength<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTagNameLength()
        }
        unsafe extern "system" fn IsTextContent<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).IsTextContent()
        }
        unsafe extern "system" fn GetParent<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, parent: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetParent(::core::mem::transmute_copy(&parent))
        }
        unsafe extern "system" fn HasChildren<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).HasChildren()
        }
        unsafe extern "system" fn GetFirstChild<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, child: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetFirstChild(::core::mem::transmute_copy(&child))
        }
        unsafe extern "system" fn GetLastChild<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, child: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetLastChild(::core::mem::transmute_copy(&child))
        }
        unsafe extern "system" fn GetPreviousChild<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, referencechild: ::windows::core::RawPtr, previouschild: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetPreviousChild(::core::mem::transmute(&referencechild)) {
                ::core::result::Result::Ok(ok__) => {
                    *previouschild = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn GetNextChild<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, referencechild: ::windows::core::RawPtr, nextchild: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetNextChild(::core::mem::transmute(&referencechild)) {
                ::core::result::Result::Ok(ok__) => {
                    *nextchild = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn InsertChildBefore<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, newchild: ::windows::core::RawPtr, referencechild: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).InsertChildBefore(::core::mem::transmute(&newchild), ::core::mem::transmute(&referencechild)).into()
        }
        unsafe extern "system" fn AppendChild<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, newchild: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AppendChild(::core::mem::transmute(&newchild)).into()
        }
        unsafe extern "system" fn ReplaceChild<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, newchild: ::windows::core::RawPtr, oldchild: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ReplaceChild(::core::mem::transmute(&newchild), ::core::mem::transmute(&oldchild)).into()
        }
        unsafe extern "system" fn RemoveChild<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, oldchild: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RemoveChild(::core::mem::transmute(&oldchild)).into()
        }
        unsafe extern "system" fn CreateChild<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, tagname: super::super::Foundation::PWSTR, newchild: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreateChild(::core::mem::transmute_copy(&tagname)) {
                ::core::result::Result::Ok(ok__) => {
                    *newchild = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        unsafe extern "system" fn IsAttributeSpecified<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, inherited: *mut super::super::Foundation::BOOL) -> super::super::Foundation::BOOL {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).IsAttributeSpecified(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&inherited))
        }
        unsafe extern "system" fn GetSpecifiedAttributeCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSpecifiedAttributeCount()
        }
        unsafe extern "system" fn GetSpecifiedAttributeName<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32, name: super::super::Foundation::PWSTR, namecount: u32, inherited: *mut super::super::Foundation::BOOL) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSpecifiedAttributeName(::core::mem::transmute_copy(&index), ::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&namecount), ::core::mem::transmute_copy(&inherited)).into()
        }
        unsafe extern "system" fn GetSpecifiedAttributeNameLength<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, index: u32, namelength: *mut u32, inherited: *mut super::super::Foundation::BOOL) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSpecifiedAttributeNameLength(::core::mem::transmute_copy(&index), ::core::mem::transmute_copy(&namelength), ::core::mem::transmute_copy(&inherited)).into()
        }
        unsafe extern "system" fn RemoveAttribute<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RemoveAttribute(::core::mem::transmute_copy(&name)).into()
        }
        unsafe extern "system" fn SetTextValue<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, namecount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetTextValue(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&namecount)).into()
        }
        unsafe extern "system" fn GetTextValue<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, namecount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTextValue(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&namecount)).into()
        }
        unsafe extern "system" fn GetTextValueLength<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTextValueLength()
        }
        unsafe extern "system" fn SetAttributeValue<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, value: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetAttributeValue(::core::mem::transmute_copy(&name), ::core::mem::transmute(&value)).into()
        }
        unsafe extern "system" fn SetAttributeValue2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, r#type: D2D1_SVG_ATTRIBUTE_POD_TYPE, value: *const ::core::ffi::c_void, valuesizeinbytes: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetAttributeValue2(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&r#type), ::core::mem::transmute_copy(&value), ::core::mem::transmute_copy(&valuesizeinbytes)).into()
        }
        unsafe extern "system" fn SetAttributeValue3<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, r#type: D2D1_SVG_ATTRIBUTE_STRING_TYPE, value: super::super::Foundation::PWSTR) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetAttributeValue3(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&r#type), ::core::mem::transmute_copy(&value)).into()
        }
        unsafe extern "system" fn GetAttributeValue<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, riid: *const ::windows::core::GUID, value: *mut *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetAttributeValue(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&riid), ::core::mem::transmute_copy(&value)).into()
        }
        unsafe extern "system" fn GetAttributeValue2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, r#type: D2D1_SVG_ATTRIBUTE_POD_TYPE, value: *mut ::core::ffi::c_void, valuesizeinbytes: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetAttributeValue2(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&r#type), ::core::mem::transmute_copy(&value), ::core::mem::transmute_copy(&valuesizeinbytes)).into()
        }
        unsafe extern "system" fn GetAttributeValue3<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, r#type: D2D1_SVG_ATTRIBUTE_STRING_TYPE, value: super::super::Foundation::PWSTR, valuecount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetAttributeValue3(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&r#type), ::core::mem::transmute_copy(&value), ::core::mem::transmute_copy(&valuecount)).into()
        }
        unsafe extern "system" fn GetAttributeValueLength<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgElement_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, name: super::super::Foundation::PWSTR, r#type: D2D1_SVG_ATTRIBUTE_STRING_TYPE, valuelength: *mut u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).GetAttributeValueLength(::core::mem::transmute_copy(&name), ::core::mem::transmute_copy(&r#type)) {
                ::core::result::Result::Ok(ok__) => {
                    *valuelength = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetDocument: GetDocument::<Identity, Impl, OFFSET>,
            GetTagName: GetTagName::<Identity, Impl, OFFSET>,
            GetTagNameLength: GetTagNameLength::<Identity, Impl, OFFSET>,
            IsTextContent: IsTextContent::<Identity, Impl, OFFSET>,
            GetParent: GetParent::<Identity, Impl, OFFSET>,
            HasChildren: HasChildren::<Identity, Impl, OFFSET>,
            GetFirstChild: GetFirstChild::<Identity, Impl, OFFSET>,
            GetLastChild: GetLastChild::<Identity, Impl, OFFSET>,
            GetPreviousChild: GetPreviousChild::<Identity, Impl, OFFSET>,
            GetNextChild: GetNextChild::<Identity, Impl, OFFSET>,
            InsertChildBefore: InsertChildBefore::<Identity, Impl, OFFSET>,
            AppendChild: AppendChild::<Identity, Impl, OFFSET>,
            ReplaceChild: ReplaceChild::<Identity, Impl, OFFSET>,
            RemoveChild: RemoveChild::<Identity, Impl, OFFSET>,
            CreateChild: CreateChild::<Identity, Impl, OFFSET>,
            IsAttributeSpecified: IsAttributeSpecified::<Identity, Impl, OFFSET>,
            GetSpecifiedAttributeCount: GetSpecifiedAttributeCount::<Identity, Impl, OFFSET>,
            GetSpecifiedAttributeName: GetSpecifiedAttributeName::<Identity, Impl, OFFSET>,
            GetSpecifiedAttributeNameLength: GetSpecifiedAttributeNameLength::<Identity, Impl, OFFSET>,
            RemoveAttribute: RemoveAttribute::<Identity, Impl, OFFSET>,
            SetTextValue: SetTextValue::<Identity, Impl, OFFSET>,
            GetTextValue: GetTextValue::<Identity, Impl, OFFSET>,
            GetTextValueLength: GetTextValueLength::<Identity, Impl, OFFSET>,
            SetAttributeValue: SetAttributeValue::<Identity, Impl, OFFSET>,
            SetAttributeValue2: SetAttributeValue2::<Identity, Impl, OFFSET>,
            SetAttributeValue3: SetAttributeValue3::<Identity, Impl, OFFSET>,
            GetAttributeValue: GetAttributeValue::<Identity, Impl, OFFSET>,
            GetAttributeValue2: GetAttributeValue2::<Identity, Impl, OFFSET>,
            GetAttributeValue3: GetAttributeValue3::<Identity, Impl, OFFSET>,
            GetAttributeValueLength: GetAttributeValueLength::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SvgElement as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1SvgGlyphStyle_Impl: Sized + ID2D1Resource_Impl {
    fn SetFill(&mut self, brush: &::core::option::Option<ID2D1Brush>) -> ::windows::core::Result<()>;
    fn GetFill(&mut self, brush: *mut ::core::option::Option<ID2D1Brush>);
    fn SetStroke(&mut self, brush: &::core::option::Option<ID2D1Brush>, strokewidth: f32, dashes: *const f32, dashescount: u32, dashoffset: f32) -> ::windows::core::Result<()>;
    fn GetStrokeDashesCount(&mut self) -> u32;
    fn GetStroke(&mut self, brush: *mut ::core::option::Option<ID2D1Brush>, strokewidth: *mut f32, dashes: *mut f32, dashescount: u32, dashoffset: *mut f32);
}
impl ID2D1SvgGlyphStyle_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgGlyphStyle_Impl, const OFFSET: isize>() -> ID2D1SvgGlyphStyle_Vtbl {
        unsafe extern "system" fn SetFill<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgGlyphStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, brush: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetFill(::core::mem::transmute(&brush)).into()
        }
        unsafe extern "system" fn GetFill<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgGlyphStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, brush: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetFill(::core::mem::transmute_copy(&brush))
        }
        unsafe extern "system" fn SetStroke<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgGlyphStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, brush: ::windows::core::RawPtr, strokewidth: f32, dashes: *const f32, dashescount: u32, dashoffset: f32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetStroke(::core::mem::transmute(&brush), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute_copy(&dashes), ::core::mem::transmute_copy(&dashescount), ::core::mem::transmute_copy(&dashoffset)).into()
        }
        unsafe extern "system" fn GetStrokeDashesCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgGlyphStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetStrokeDashesCount()
        }
        unsafe extern "system" fn GetStroke<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgGlyphStyle_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, brush: *mut ::windows::core::RawPtr, strokewidth: *mut f32, dashes: *mut f32, dashescount: u32, dashoffset: *mut f32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetStroke(::core::mem::transmute_copy(&brush), ::core::mem::transmute_copy(&strokewidth), ::core::mem::transmute_copy(&dashes), ::core::mem::transmute_copy(&dashescount), ::core::mem::transmute_copy(&dashoffset))
        }
        Self {
            base: ID2D1Resource_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetFill: SetFill::<Identity, Impl, OFFSET>,
            GetFill: GetFill::<Identity, Impl, OFFSET>,
            SetStroke: SetStroke::<Identity, Impl, OFFSET>,
            GetStrokeDashesCount: GetStrokeDashesCount::<Identity, Impl, OFFSET>,
            GetStroke: GetStroke::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SvgGlyphStyle as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1SvgPaint_Impl: Sized + ID2D1Resource_Impl + ID2D1SvgAttribute_Impl {
    fn SetPaintType(&mut self, painttype: D2D1_SVG_PAINT_TYPE) -> ::windows::core::Result<()>;
    fn GetPaintType(&mut self) -> D2D1_SVG_PAINT_TYPE;
    fn SetColor(&mut self, color: *const Common::D2D1_COLOR_F) -> ::windows::core::Result<()>;
    fn GetColor(&mut self, color: *mut Common::D2D1_COLOR_F);
    fn SetId(&mut self, id: super::super::Foundation::PWSTR) -> ::windows::core::Result<()>;
    fn GetId(&mut self, id: super::super::Foundation::PWSTR, idcount: u32) -> ::windows::core::Result<()>;
    fn GetIdLength(&mut self) -> u32;
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1SvgPaint_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPaint_Impl, const OFFSET: isize>() -> ID2D1SvgPaint_Vtbl {
        unsafe extern "system" fn SetPaintType<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPaint_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, painttype: D2D1_SVG_PAINT_TYPE) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetPaintType(::core::mem::transmute_copy(&painttype)).into()
        }
        unsafe extern "system" fn GetPaintType<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPaint_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> D2D1_SVG_PAINT_TYPE {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPaintType()
        }
        unsafe extern "system" fn SetColor<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPaint_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, color: *const Common::D2D1_COLOR_F) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetColor(::core::mem::transmute_copy(&color)).into()
        }
        unsafe extern "system" fn GetColor<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPaint_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, color: *mut Common::D2D1_COLOR_F) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetColor(::core::mem::transmute_copy(&color))
        }
        unsafe extern "system" fn SetId<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPaint_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, id: super::super::Foundation::PWSTR) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetId(::core::mem::transmute_copy(&id)).into()
        }
        unsafe extern "system" fn GetId<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPaint_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, id: super::super::Foundation::PWSTR, idcount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetId(::core::mem::transmute_copy(&id), ::core::mem::transmute_copy(&idcount)).into()
        }
        unsafe extern "system" fn GetIdLength<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPaint_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetIdLength()
        }
        Self {
            base: ID2D1SvgAttribute_Vtbl::new::<Identity, Impl, OFFSET>(),
            SetPaintType: SetPaintType::<Identity, Impl, OFFSET>,
            GetPaintType: GetPaintType::<Identity, Impl, OFFSET>,
            SetColor: SetColor::<Identity, Impl, OFFSET>,
            GetColor: GetColor::<Identity, Impl, OFFSET>,
            SetId: SetId::<Identity, Impl, OFFSET>,
            GetId: GetId::<Identity, Impl, OFFSET>,
            GetIdLength: GetIdLength::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SvgPaint as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1SvgAttribute as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
pub trait ID2D1SvgPathData_Impl: Sized + ID2D1Resource_Impl + ID2D1SvgAttribute_Impl {
    fn RemoveSegmentDataAtEnd(&mut self, datacount: u32) -> ::windows::core::Result<()>;
    fn UpdateSegmentData(&mut self, data: *const f32, datacount: u32, startindex: u32) -> ::windows::core::Result<()>;
    fn GetSegmentData(&mut self, data: *mut f32, datacount: u32, startindex: u32) -> ::windows::core::Result<()>;
    fn GetSegmentDataCount(&mut self) -> u32;
    fn RemoveCommandsAtEnd(&mut self, commandscount: u32) -> ::windows::core::Result<()>;
    fn UpdateCommands(&mut self, commands: *const D2D1_SVG_PATH_COMMAND, commandscount: u32, startindex: u32) -> ::windows::core::Result<()>;
    fn GetCommands(&mut self, commands: *mut D2D1_SVG_PATH_COMMAND, commandscount: u32, startindex: u32) -> ::windows::core::Result<()>;
    fn GetCommandsCount(&mut self) -> u32;
    fn CreatePathGeometry(&mut self, fillmode: Common::D2D1_FILL_MODE) -> ::windows::core::Result<ID2D1PathGeometry1>;
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
impl ID2D1SvgPathData_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPathData_Impl, const OFFSET: isize>() -> ID2D1SvgPathData_Vtbl {
        unsafe extern "system" fn RemoveSegmentDataAtEnd<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPathData_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, datacount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RemoveSegmentDataAtEnd(::core::mem::transmute_copy(&datacount)).into()
        }
        unsafe extern "system" fn UpdateSegmentData<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPathData_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, data: *const f32, datacount: u32, startindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).UpdateSegmentData(::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&datacount), ::core::mem::transmute_copy(&startindex)).into()
        }
        unsafe extern "system" fn GetSegmentData<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPathData_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, data: *mut f32, datacount: u32, startindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSegmentData(::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&datacount), ::core::mem::transmute_copy(&startindex)).into()
        }
        unsafe extern "system" fn GetSegmentDataCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPathData_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSegmentDataCount()
        }
        unsafe extern "system" fn RemoveCommandsAtEnd<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPathData_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, commandscount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RemoveCommandsAtEnd(::core::mem::transmute_copy(&commandscount)).into()
        }
        unsafe extern "system" fn UpdateCommands<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPathData_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, commands: *const D2D1_SVG_PATH_COMMAND, commandscount: u32, startindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).UpdateCommands(::core::mem::transmute_copy(&commands), ::core::mem::transmute_copy(&commandscount), ::core::mem::transmute_copy(&startindex)).into()
        }
        unsafe extern "system" fn GetCommands<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPathData_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, commands: *mut D2D1_SVG_PATH_COMMAND, commandscount: u32, startindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetCommands(::core::mem::transmute_copy(&commands), ::core::mem::transmute_copy(&commandscount), ::core::mem::transmute_copy(&startindex)).into()
        }
        unsafe extern "system" fn GetCommandsCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPathData_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetCommandsCount()
        }
        unsafe extern "system" fn CreatePathGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPathData_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, fillmode: Common::D2D1_FILL_MODE, pathgeometry: *mut ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).CreatePathGeometry(::core::mem::transmute_copy(&fillmode)) {
                ::core::result::Result::Ok(ok__) => {
                    *pathgeometry = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1SvgAttribute_Vtbl::new::<Identity, Impl, OFFSET>(),
            RemoveSegmentDataAtEnd: RemoveSegmentDataAtEnd::<Identity, Impl, OFFSET>,
            UpdateSegmentData: UpdateSegmentData::<Identity, Impl, OFFSET>,
            GetSegmentData: GetSegmentData::<Identity, Impl, OFFSET>,
            GetSegmentDataCount: GetSegmentDataCount::<Identity, Impl, OFFSET>,
            RemoveCommandsAtEnd: RemoveCommandsAtEnd::<Identity, Impl, OFFSET>,
            UpdateCommands: UpdateCommands::<Identity, Impl, OFFSET>,
            GetCommands: GetCommands::<Identity, Impl, OFFSET>,
            GetCommandsCount: GetCommandsCount::<Identity, Impl, OFFSET>,
            CreatePathGeometry: CreatePathGeometry::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SvgPathData as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1SvgAttribute as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
pub trait ID2D1SvgPointCollection_Impl: Sized + ID2D1Resource_Impl + ID2D1SvgAttribute_Impl {
    fn RemovePointsAtEnd(&mut self, pointscount: u32) -> ::windows::core::Result<()>;
    fn UpdatePoints(&mut self, points: *const Common::D2D_POINT_2F, pointscount: u32, startindex: u32) -> ::windows::core::Result<()>;
    fn GetPoints(&mut self, points: *mut Common::D2D_POINT_2F, pointscount: u32, startindex: u32) -> ::windows::core::Result<()>;
    fn GetPointsCount(&mut self) -> u32;
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
impl ID2D1SvgPointCollection_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPointCollection_Impl, const OFFSET: isize>() -> ID2D1SvgPointCollection_Vtbl {
        unsafe extern "system" fn RemovePointsAtEnd<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPointCollection_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, pointscount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RemovePointsAtEnd(::core::mem::transmute_copy(&pointscount)).into()
        }
        unsafe extern "system" fn UpdatePoints<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPointCollection_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, points: *const Common::D2D_POINT_2F, pointscount: u32, startindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).UpdatePoints(::core::mem::transmute_copy(&points), ::core::mem::transmute_copy(&pointscount), ::core::mem::transmute_copy(&startindex)).into()
        }
        unsafe extern "system" fn GetPoints<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPointCollection_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, points: *mut Common::D2D_POINT_2F, pointscount: u32, startindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPoints(::core::mem::transmute_copy(&points), ::core::mem::transmute_copy(&pointscount), ::core::mem::transmute_copy(&startindex)).into()
        }
        unsafe extern "system" fn GetPointsCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgPointCollection_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetPointsCount()
        }
        Self {
            base: ID2D1SvgAttribute_Vtbl::new::<Identity, Impl, OFFSET>(),
            RemovePointsAtEnd: RemovePointsAtEnd::<Identity, Impl, OFFSET>,
            UpdatePoints: UpdatePoints::<Identity, Impl, OFFSET>,
            GetPoints: GetPoints::<Identity, Impl, OFFSET>,
            GetPointsCount: GetPointsCount::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SvgPointCollection as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1SvgAttribute as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1SvgStrokeDashArray_Impl: Sized + ID2D1Resource_Impl + ID2D1SvgAttribute_Impl {
    fn RemoveDashesAtEnd(&mut self, dashescount: u32) -> ::windows::core::Result<()>;
    fn UpdateDashes(&mut self, dashes: *const D2D1_SVG_LENGTH, dashescount: u32, startindex: u32) -> ::windows::core::Result<()>;
    fn UpdateDashes2(&mut self, dashes: *const f32, dashescount: u32, startindex: u32) -> ::windows::core::Result<()>;
    fn GetDashes(&mut self, dashes: *mut D2D1_SVG_LENGTH, dashescount: u32, startindex: u32) -> ::windows::core::Result<()>;
    fn GetDashes2(&mut self, dashes: *mut f32, dashescount: u32, startindex: u32) -> ::windows::core::Result<()>;
    fn GetDashesCount(&mut self) -> u32;
}
impl ID2D1SvgStrokeDashArray_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgStrokeDashArray_Impl, const OFFSET: isize>() -> ID2D1SvgStrokeDashArray_Vtbl {
        unsafe extern "system" fn RemoveDashesAtEnd<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgStrokeDashArray_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dashescount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RemoveDashesAtEnd(::core::mem::transmute_copy(&dashescount)).into()
        }
        unsafe extern "system" fn UpdateDashes<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgStrokeDashArray_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dashes: *const D2D1_SVG_LENGTH, dashescount: u32, startindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).UpdateDashes(::core::mem::transmute_copy(&dashes), ::core::mem::transmute_copy(&dashescount), ::core::mem::transmute_copy(&startindex)).into()
        }
        unsafe extern "system" fn UpdateDashes2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgStrokeDashArray_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dashes: *const f32, dashescount: u32, startindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).UpdateDashes2(::core::mem::transmute_copy(&dashes), ::core::mem::transmute_copy(&dashescount), ::core::mem::transmute_copy(&startindex)).into()
        }
        unsafe extern "system" fn GetDashes<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgStrokeDashArray_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dashes: *mut D2D1_SVG_LENGTH, dashescount: u32, startindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDashes(::core::mem::transmute_copy(&dashes), ::core::mem::transmute_copy(&dashescount), ::core::mem::transmute_copy(&startindex)).into()
        }
        unsafe extern "system" fn GetDashes2<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgStrokeDashArray_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, dashes: *mut f32, dashescount: u32, startindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDashes2(::core::mem::transmute_copy(&dashes), ::core::mem::transmute_copy(&dashescount), ::core::mem::transmute_copy(&startindex)).into()
        }
        unsafe extern "system" fn GetDashesCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1SvgStrokeDashArray_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetDashesCount()
        }
        Self {
            base: ID2D1SvgAttribute_Vtbl::new::<Identity, Impl, OFFSET>(),
            RemoveDashesAtEnd: RemoveDashesAtEnd::<Identity, Impl, OFFSET>,
            UpdateDashes: UpdateDashes::<Identity, Impl, OFFSET>,
            UpdateDashes2: UpdateDashes2::<Identity, Impl, OFFSET>,
            GetDashes: GetDashes::<Identity, Impl, OFFSET>,
            GetDashes2: GetDashes2::<Identity, Impl, OFFSET>,
            GetDashesCount: GetDashesCount::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1SvgStrokeDashArray as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1SvgAttribute as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
pub trait ID2D1TessellationSink_Impl: Sized {
    fn AddTriangles(&mut self, triangles: *const D2D1_TRIANGLE, trianglescount: u32);
    fn Close(&mut self) -> ::windows::core::Result<()>;
}
#[cfg(feature = "Win32_Graphics_Direct2D_Common")]
impl ID2D1TessellationSink_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TessellationSink_Impl, const OFFSET: isize>() -> ID2D1TessellationSink_Vtbl {
        unsafe extern "system" fn AddTriangles<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TessellationSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, triangles: *const D2D1_TRIANGLE, trianglescount: u32) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AddTriangles(::core::mem::transmute_copy(&triangles), ::core::mem::transmute_copy(&trianglescount))
        }
        unsafe extern "system" fn Close<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TessellationSink_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Close().into()
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            AddTriangles: AddTriangles::<Identity, Impl, OFFSET>,
            Close: Close::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1TessellationSink as ::windows::core::Interface>::IID
    }
}
#[cfg(feature = "Win32_Foundation")]
pub trait ID2D1Transform_Impl: Sized + ID2D1TransformNode_Impl {
    fn MapOutputRectToInputRects(&mut self, outputrect: *const super::super::Foundation::RECT, inputrects: *mut super::super::Foundation::RECT, inputrectscount: u32) -> ::windows::core::Result<()>;
    fn MapInputRectsToOutputRect(&mut self, inputrects: *const super::super::Foundation::RECT, inputopaquesubrects: *const super::super::Foundation::RECT, inputrectcount: u32, outputrect: *mut super::super::Foundation::RECT, outputopaquesubrect: *mut super::super::Foundation::RECT) -> ::windows::core::Result<()>;
    fn MapInvalidRect(&mut self, inputindex: u32, invalidinputrect: &super::super::Foundation::RECT) -> ::windows::core::Result<super::super::Foundation::RECT>;
}
#[cfg(feature = "Win32_Foundation")]
impl ID2D1Transform_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Transform_Impl, const OFFSET: isize>() -> ID2D1Transform_Vtbl {
        unsafe extern "system" fn MapOutputRectToInputRects<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Transform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, outputrect: *const super::super::Foundation::RECT, inputrects: *mut super::super::Foundation::RECT, inputrectscount: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).MapOutputRectToInputRects(::core::mem::transmute_copy(&outputrect), ::core::mem::transmute_copy(&inputrects), ::core::mem::transmute_copy(&inputrectscount)).into()
        }
        unsafe extern "system" fn MapInputRectsToOutputRect<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Transform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inputrects: *const super::super::Foundation::RECT, inputopaquesubrects: *const super::super::Foundation::RECT, inputrectcount: u32, outputrect: *mut super::super::Foundation::RECT, outputopaquesubrect: *mut super::super::Foundation::RECT) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).MapInputRectsToOutputRect(::core::mem::transmute_copy(&inputrects), ::core::mem::transmute_copy(&inputopaquesubrects), ::core::mem::transmute_copy(&inputrectcount), ::core::mem::transmute_copy(&outputrect), ::core::mem::transmute_copy(&outputopaquesubrect)).into()
        }
        unsafe extern "system" fn MapInvalidRect<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1Transform_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, inputindex: u32, invalidinputrect: super::super::Foundation::RECT, invalidoutputrect: *mut super::super::Foundation::RECT) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            match (*this).MapInvalidRect(::core::mem::transmute_copy(&inputindex), ::core::mem::transmute_copy(&invalidinputrect)) {
                ::core::result::Result::Ok(ok__) => {
                    *invalidoutputrect = ::core::mem::transmute(ok__);
                    ::windows::core::HRESULT(0)
                }
                ::core::result::Result::Err(err) => err.into(),
            }
        }
        Self {
            base: ID2D1TransformNode_Vtbl::new::<Identity, Impl, OFFSET>(),
            MapOutputRectToInputRects: MapOutputRectToInputRects::<Identity, Impl, OFFSET>,
            MapInputRectsToOutputRect: MapInputRectsToOutputRect::<Identity, Impl, OFFSET>,
            MapInvalidRect: MapInvalidRect::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1Transform as ::windows::core::Interface>::IID || iid == &<ID2D1TransformNode as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1TransformGraph_Impl: Sized {
    fn GetInputCount(&mut self) -> u32;
    fn SetSingleTransformNode(&mut self, node: &::core::option::Option<ID2D1TransformNode>) -> ::windows::core::Result<()>;
    fn AddNode(&mut self, node: &::core::option::Option<ID2D1TransformNode>) -> ::windows::core::Result<()>;
    fn RemoveNode(&mut self, node: &::core::option::Option<ID2D1TransformNode>) -> ::windows::core::Result<()>;
    fn SetOutputNode(&mut self, node: &::core::option::Option<ID2D1TransformNode>) -> ::windows::core::Result<()>;
    fn ConnectNode(&mut self, fromnode: &::core::option::Option<ID2D1TransformNode>, tonode: &::core::option::Option<ID2D1TransformNode>, tonodeinputindex: u32) -> ::windows::core::Result<()>;
    fn ConnectToEffectInput(&mut self, toeffectinputindex: u32, node: &::core::option::Option<ID2D1TransformNode>, tonodeinputindex: u32) -> ::windows::core::Result<()>;
    fn Clear(&mut self);
    fn SetPassthroughGraph(&mut self, effectinputindex: u32) -> ::windows::core::Result<()>;
}
impl ID2D1TransformGraph_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformGraph_Impl, const OFFSET: isize>() -> ID2D1TransformGraph_Vtbl {
        unsafe extern "system" fn GetInputCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformGraph_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetInputCount()
        }
        unsafe extern "system" fn SetSingleTransformNode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformGraph_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, node: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetSingleTransformNode(::core::mem::transmute(&node)).into()
        }
        unsafe extern "system" fn AddNode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformGraph_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, node: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).AddNode(::core::mem::transmute(&node)).into()
        }
        unsafe extern "system" fn RemoveNode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformGraph_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, node: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).RemoveNode(::core::mem::transmute(&node)).into()
        }
        unsafe extern "system" fn SetOutputNode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformGraph_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, node: ::windows::core::RawPtr) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetOutputNode(::core::mem::transmute(&node)).into()
        }
        unsafe extern "system" fn ConnectNode<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformGraph_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, fromnode: ::windows::core::RawPtr, tonode: ::windows::core::RawPtr, tonodeinputindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ConnectNode(::core::mem::transmute(&fromnode), ::core::mem::transmute(&tonode), ::core::mem::transmute_copy(&tonodeinputindex)).into()
        }
        unsafe extern "system" fn ConnectToEffectInput<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformGraph_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, toeffectinputindex: u32, node: ::windows::core::RawPtr, tonodeinputindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).ConnectToEffectInput(::core::mem::transmute_copy(&toeffectinputindex), ::core::mem::transmute(&node), ::core::mem::transmute_copy(&tonodeinputindex)).into()
        }
        unsafe extern "system" fn Clear<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformGraph_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Clear()
        }
        unsafe extern "system" fn SetPassthroughGraph<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformGraph_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, effectinputindex: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).SetPassthroughGraph(::core::mem::transmute_copy(&effectinputindex)).into()
        }
        Self {
            base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(),
            GetInputCount: GetInputCount::<Identity, Impl, OFFSET>,
            SetSingleTransformNode: SetSingleTransformNode::<Identity, Impl, OFFSET>,
            AddNode: AddNode::<Identity, Impl, OFFSET>,
            RemoveNode: RemoveNode::<Identity, Impl, OFFSET>,
            SetOutputNode: SetOutputNode::<Identity, Impl, OFFSET>,
            ConnectNode: ConnectNode::<Identity, Impl, OFFSET>,
            ConnectToEffectInput: ConnectToEffectInput::<Identity, Impl, OFFSET>,
            Clear: Clear::<Identity, Impl, OFFSET>,
            SetPassthroughGraph: SetPassthroughGraph::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1TransformGraph as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1TransformNode_Impl: Sized {
    fn GetInputCount(&mut self) -> u32;
}
impl ID2D1TransformNode_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformNode_Impl, const OFFSET: isize>() -> ID2D1TransformNode_Vtbl {
        unsafe extern "system" fn GetInputCount<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformNode_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> u32 {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetInputCount()
        }
        Self { base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(), GetInputCount: GetInputCount::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1TransformNode as ::windows::core::Interface>::IID
    }
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
pub trait ID2D1TransformedGeometry_Impl: Sized + ID2D1Resource_Impl + ID2D1Geometry_Impl {
    fn GetSourceGeometry(&mut self, sourcegeometry: *mut ::core::option::Option<ID2D1Geometry>);
    fn GetTransform(&mut self, transform: *mut super::super::super::Foundation::Numerics::Matrix3x2);
}
#[cfg(all(feature = "Foundation_Numerics", feature = "Win32_Foundation", feature = "Win32_Graphics_Direct2D_Common"))]
impl ID2D1TransformedGeometry_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformedGeometry_Impl, const OFFSET: isize>() -> ID2D1TransformedGeometry_Vtbl {
        unsafe extern "system" fn GetSourceGeometry<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformedGeometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, sourcegeometry: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSourceGeometry(::core::mem::transmute_copy(&sourcegeometry))
        }
        unsafe extern "system" fn GetTransform<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformedGeometry_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, transform: *mut super::super::super::Foundation::Numerics::Matrix3x2) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetTransform(::core::mem::transmute_copy(&transform))
        }
        Self {
            base: ID2D1Geometry_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetSourceGeometry: GetSourceGeometry::<Identity, Impl, OFFSET>,
            GetTransform: GetTransform::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1TransformedGeometry as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Geometry as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1TransformedImageSource_Impl: Sized + ID2D1Resource_Impl + ID2D1Image_Impl {
    fn GetSource(&mut self, imagesource: *mut ::core::option::Option<ID2D1ImageSource>);
    fn GetProperties(&mut self, properties: *mut D2D1_TRANSFORMED_IMAGE_SOURCE_PROPERTIES);
}
impl ID2D1TransformedImageSource_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformedImageSource_Impl, const OFFSET: isize>() -> ID2D1TransformedImageSource_Vtbl {
        unsafe extern "system" fn GetSource<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformedImageSource_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, imagesource: *mut ::windows::core::RawPtr) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetSource(::core::mem::transmute_copy(&imagesource))
        }
        unsafe extern "system" fn GetProperties<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1TransformedImageSource_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, properties: *mut D2D1_TRANSFORMED_IMAGE_SOURCE_PROPERTIES) {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).GetProperties(::core::mem::transmute_copy(&properties))
        }
        Self {
            base: ID2D1Image_Vtbl::new::<Identity, Impl, OFFSET>(),
            GetSource: GetSource::<Identity, Impl, OFFSET>,
            GetProperties: GetProperties::<Identity, Impl, OFFSET>,
        }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1TransformedImageSource as ::windows::core::Interface>::IID || iid == &<ID2D1Resource as ::windows::core::Interface>::IID || iid == &<ID2D1Image as ::windows::core::Interface>::IID
    }
}
pub trait ID2D1VertexBuffer_Impl: Sized {
    fn Map(&mut self, data: *mut *mut u8, buffersize: u32) -> ::windows::core::Result<()>;
    fn Unmap(&mut self) -> ::windows::core::Result<()>;
}
impl ID2D1VertexBuffer_Vtbl {
    pub const fn new<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1VertexBuffer_Impl, const OFFSET: isize>() -> ID2D1VertexBuffer_Vtbl {
        unsafe extern "system" fn Map<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1VertexBuffer_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void, data: *mut *mut u8, buffersize: u32) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Map(::core::mem::transmute_copy(&data), ::core::mem::transmute_copy(&buffersize)).into()
        }
        unsafe extern "system" fn Unmap<Identity: ::windows::core::IUnknownImpl, Impl: ID2D1VertexBuffer_Impl, const OFFSET: isize>(this: *mut ::core::ffi::c_void) -> ::windows::core::HRESULT {
            let this = (this as *mut ::windows::core::RawPtr).offset(OFFSET) as *mut Identity;
            let this = (*this).get_impl() as *mut Impl;
            (*this).Unmap().into()
        }
        Self { base: ::windows::core::IUnknownVtbl::new::<Identity, OFFSET>(), Map: Map::<Identity, Impl, OFFSET>, Unmap: Unmap::<Identity, Impl, OFFSET> }
    }
    pub fn matches(iid: &windows::core::GUID) -> bool {
        iid == &<ID2D1VertexBuffer as ::windows::core::Interface>::IID
    }
}

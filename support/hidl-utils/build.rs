extern crate cc;

use std::env;
use std::path::Path;

fn main() {
    if let Ok(_) = env::var("BUILD_WITH_NDK_DIR") {
        let path = env::var("CARGO_MANIFEST_DIR").unwrap();
        println!(
            "cargo:rustc-link-search=native={}",
            Path::new(&path).join("libnative").join(env::var("TARGET").unwrap()).display()
        );

        println!("cargo:rustc-link-lib=dylib=hidlbase");
        println!("cargo:rustc-link-lib=dylib=hidltransport");
        println!("cargo:rustc-link-lib=dylib=utils");
        println!("cargo:rustc-link-lib=dylib=binder");
        println!("cargo:rustc-link-lib=dylib=hwbinder");
        println!("cargo:rustc-link-lib=dylib=c++_shared");
        if let Ok(_) = env::var("GONK_DIR") {

            #[cfg(target_arch = "arm")]
            let asm_dir = "asm-arm";
            #[cfg(target_arch = "aarch64")]
            let asm_dir = "asm-arm64";
            #[cfg(target_arch = "x86_64")]
            let asm_dir = "asm-x86";

            let _prefix = env::var("GONK_DIR").expect("Please set the GONK_DIR env variable");
            let _product =
                env::var("GONK_PRODUCT").expect("Please set the GONK_PRODUCT env variable");
            let _sysroot = format!("--sysroot={}/out/target/product/{}/obj", _prefix, _product);
            let _hidl_base_inc = format!("{}/system/libhidl/base/include", _prefix);
            let _cutils_inc = format!("{}/system/core/libcutils/include", _prefix);
            let _utils_inc = format!("{}/system/core/libutils/include", _prefix);
            let _libbacktrace_inc = format!("{}/system/core/libbacktrace/include", _prefix);
            let _liblog_inc = format!("{}/system/core/liblog/include", _prefix);
            let _libsystem_inc = format!("{}/system/core/libsystem/include", _prefix);
            let _hidl_trans_inc = format!("{}//system/libhidl/transport/include", _prefix);
            let _core_base_inc = format!("{}/system/core/base/include", _prefix);
            let _trans_manager_inc = format!("{}/out/soong/.intermediates/system/libhidl/transport/manager/1.0/android.hidl.manager@1.0_genc++_headers/gen", _prefix);
            let _trans_base_inc = format!("{}/out/soong/.intermediates/system/libhidl/transport/base/1.0/android.hidl.base@1.0_genc++_headers/gen", _prefix);
            let _hwbinder_inc = format!("{}/system/libhwbinder/include", _prefix);
            let _libcxx_inc = format!("{}/external/libcxx/include", _prefix);
            let _libcxxabi_inc = format!("{}/external/libcxxabi/include", _prefix);
            let _sys_core_inc = format!("{}/system/core/include", _prefix);
            let _sys_media_audio_inc = format!("{}/system/media/audio/include", _prefix);
            let _libhw_inc = format!("{}/hardware/libhardware/include", _prefix);
            let _libhw_legacy_inc = format!("{}/hardware/libhardware_legacy/include", _prefix);
            let _ril_inc = format!("{}/hardware/ril/include", _prefix);
            let _libnative_inc = format!("{}/libnativehelper/include", _prefix);
            let _frameworks_native_inc = format!("{}/frameworks/native/include", _prefix);
            let _frameworks_native_gl_inc = format!("{}/frameworks/native/opengl/include", _prefix);
            let _frameworks_native_av_inc = format!("{}/frameworks/av/include", _prefix);
            let _libc_inc = format!("{}/bionic/libc/include", _prefix);
            let _libc_kernel_inc = format!("{}/bionic/libc/kernel/uapi", _prefix);
            let _libc_kernel_arch_inc = format!("{}/bionic/libc/kernel/uapi/{}", _prefix, asm_dir);
            let _libc_kernel_scsi_inc = format!("{}/bionic/libc/kernel/android/scsi", _prefix);
            let _libc_kernel_android_uapi = format!("{}/bionic/libc/kernel/android/uapi", _prefix);

            cc::Build::new()
                .file("parcel.cpp")
                .file("ibinder.cpp")
                .include(_hidl_base_inc)
                .include(_cutils_inc)
                .include(_utils_inc)
                .include(_libbacktrace_inc)
                .include(_liblog_inc)
                .include(_libsystem_inc)
                .include(_hidl_trans_inc)
                .include(_core_base_inc)
                .include(_trans_manager_inc)
                .include(_trans_base_inc)
                .include(_hwbinder_inc)
                .include(_libcxx_inc)
                .include(_libcxxabi_inc)
                .include(_sys_core_inc)
                .include(_sys_media_audio_inc)
                .include(_libhw_inc)
                .include(_libhw_legacy_inc)
                .include(_ril_inc)
                .include(_libnative_inc)
                .include(_frameworks_native_inc)
                .include(_frameworks_native_gl_inc)
                .include(_frameworks_native_av_inc)
                .include(_libc_inc)
                .include(_libc_kernel_inc)
                .include(_libc_kernel_arch_inc)
                .include(_libc_kernel_scsi_inc)
                .include(_libc_kernel_android_uapi)
                .flag(&_sysroot.to_string())
                .compile("utils-c");
        }
    }
}

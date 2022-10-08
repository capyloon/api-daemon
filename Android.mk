#
# Glue to call the cargo based build system.
#

LOCAL_PATH:= $(call my-dir)
GONK_DIR := $(abspath $(LOCAL_PATH)/../../)
API_DAEMON_ROOT := $(abspath $(LOCAL_PATH))

# Add the api-daemon executable.
include $(CLEAR_VARS)

RUST_TARGET := armv7-linux-androideabi
TARGET_INCLUDE := arm-linux-androideabi

ifeq ($(TARGET_ARCH),x86_64)
RUST_TARGET := x86_64-linux-android
TARGET_INCLUDE := $(RUST_TARGET)
LIBSUFFIX := 64
endif

ifeq ($(TARGET_ARCH),arm64)
RUST_TARGET := aarch64-linux-android
TARGET_INCLUDE := $(RUST_TARGET)
LIBSUFFIX := 64
endif

API_DAEMON_EXEC := prebuilts/$(RUST_TARGET)/api-daemon

LOCAL_MODULE := api-daemon
LOCAL_MODULE_CLASS := EXECUTABLES
LOCAL_MODULE_TAGS := optional
LOCAL_SHARED_LIBRARIES := libc libm libdl liblog libssl libcutils libc++_shared
LOCAL_MODULE_PATH := $(TARGET_OUT)/api-daemon

API_DAEMON_LIB_DEPS := \
	libhwbinder.so \
	libhidlbase.so \
	libvndksupport.so \
	libcrypto.so \
	libselinux.so \
	libpower.so \
	librecovery.so \
	$(NULL)

include $(BUILD_SYSTEM)/base_rules.mk

ifndef ANDROID_NDK
LOCAL_NDK := $(HOME)/.mozbuild/android-ndk-r25b
else
LOCAL_NDK := $(ANDROID_NDK)
endif

$(LOCAL_BUILT_MODULE): $(TARGET_CRTBEGIN_DYNAMIC_O) $(TARGET_CRTEND_O) $(addprefix $(TARGET_OUT_SHARED_LIBRARIES)/,$(API_DAEMON_LIB_DEPS))
	@echo "api-daemon: $(API_DAEMON_EXEC)"
	export TARGET_ARCH=$(RUST_TARGET) && \
	export BUILD_WITH_NDK_DIR=$(LOCAL_NDK) && \
	export GONK_DIR=$(GONK_DIR) && \
	export GONK_PRODUCT=$(TARGET_DEVICE) && \
	(cd $(API_DAEMON_ROOT) ; $(SHELL) update-prebuilts.sh)
	@touch $(TARGET_OUT_INTERMEDIATES)/EXECUTABLES/api-daemon_intermediates/api-daemon
	@mkdir -p $(@D)
	@mkdir -p $(TARGET_OUT)/b2g/defaults
	@mkdir -p $(TARGET_OUT)/api-daemon
	@rm -rf $(TARGET_OUT)/api-daemon/*

	@cp $(API_DAEMON_ROOT)/daemon/config-device.toml $(TARGET_OUT)/api-daemon/config.toml
	@cp -R $(API_DAEMON_ROOT)/prebuilts/http_root $(TARGET_OUT)/api-daemon/
	@cp $(API_DAEMON_ROOT)/$(API_DAEMON_EXEC) $(TARGET_OUT)/bin/
	@cp $(API_DAEMON_ROOT)/services/devicecapability/devicecapability.json $(TARGET_OUT)/b2g/defaults/devicecapability.json
	@cp $(LOCAL_NDK)/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/$(TARGET_INCLUDE)/libc++_shared.so $(TARGET_OUT)/lib$(LIBSUFFIX)

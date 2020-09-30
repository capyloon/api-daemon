/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
/* vim: set ts=8 sts=4 et sw=4 tw=80: */
#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include <android/hidl/manager/1.0/IServiceManager.h>
#include <hidl/HidlBinderSupport.h>
#include <hidl/HidlTransportSupport.h>
#include <hidl/HidlTransportUtils.h>
#include <hidl/ServiceManagement.h>

#define __DECL_C_START extern "C" {
#define __DECL_C_END }


struct Parcel {
    int mDummy;
    ::android::hardware::Parcel mParcel;
};

__DECL_C_START

Parcel*
parcel_create() {
    auto parcel = new Parcel;
    return parcel;
}

void
parcel_delete(Parcel* aParcel) {
    delete aParcel;
}

#define READ_WRITE_FORWARD(X, Y, T)                                        \
    int parcel_write_##Y(Parcel* aParcel, T aVal) {                      \
        ::android::status_t status = aParcel->mParcel.write##X(aVal);   \
        return (status != ::android::OK);                               \
    }                                                                   \
    int parcel_read_##Y(Parcel* aParcel, T* aVal) {                      \
        ::android::status_t status = aParcel->mParcel.read##X(aVal);    \
        return (status != ::android::OK);                               \
    }

int
parcel_write_interface_token(Parcel* aParcel, const char* aIface, size_t aLen) {
    auto buf = new char[aLen + 1];
    memcpy(buf, aIface, aLen);
    buf[aLen] = 0;
    ::android::status_t status = aParcel->mParcel.writeInterfaceToken(buf);
    delete [] buf;
    return status != ::android::OK;
}

int
parcel_write(Parcel* aParcel, const void* aData, size_t aLen) {
    ::android::status_t status = aParcel->mParcel.write(aData, aLen);
    return status != ::android::OK;
}

int
parcel_read(Parcel* aParcel, void* aData, size_t aLen) {
    ::android::status_t status = aParcel->mParcel.read(aData, aLen);
    return status != ::android::OK;
}

READ_WRITE_FORWARD(Int8, int8, int8_t);
READ_WRITE_FORWARD(Uint8, uint8, uint8_t);
READ_WRITE_FORWARD(Int16, int16, int16_t);
READ_WRITE_FORWARD(Uint16, uint16, uint16_t);
READ_WRITE_FORWARD(Int32, int32, int32_t);
READ_WRITE_FORWARD(Uint32, uint32, uint32_t);
READ_WRITE_FORWARD(Int64, int64, int64_t);
READ_WRITE_FORWARD(Uint64, uint64, uint64_t);
READ_WRITE_FORWARD(Float, float, float);
READ_WRITE_FORWARD(Double, double, double);
READ_WRITE_FORWARD(Bool, bool, bool);

int
parcel_writeCString(Parcel* aParcel, const char* aData) {
    ::android::status_t status = aParcel->mParcel.writeCString(aData);
    return status != ::android::OK;
}

const char*
parcel_readCString(Parcel* aParcel) {
    return aParcel->mParcel.readCString();
}

int
parcel_writeString16(Parcel* aParcel, const char16_t* aData, size_t aLen) {
    ::android::status_t status = aParcel->mParcel.writeString16(aData, aLen);
    return status != ::android::OK;
}

const char16_t*
parcel_readString16(Parcel* aParcel, size_t* aOutLen) {
    return aParcel->mParcel.readString16Inplace(aOutLen);
}

int
parcel_read_buffer(Parcel* aParcel, size_t aBufferSize,
                  size_t* aBufferHandle, char *aBuffer) {
    const void *out;
    ::android::status_t status =
          aParcel->mParcel.readBuffer(aBufferSize, aBufferHandle, &out);
    if (status != ::android::OK || out == nullptr) {
        return 1;
    }
    memcpy(aBuffer, out, aBufferSize);
    return 0;
}

int
parcel_write_buffer(Parcel* aParcel, char *aBuffer,
                   size_t aBufferSize,
                   size_t* aBufferHandle) {
    ::android::status_t status =
          aParcel->mParcel.writeBuffer(aBuffer, aBufferSize, aBufferHandle);
    if (status != ::android::OK) {
        return 1;
    }
    return 0;
}

int
parcel_write_embedded_buffer(Parcel* aParcel,
                           char *aBuffer, size_t aBufferSize,
                           size_t* aBufferHandle,
                           size_t aParentBufferHandle, size_t aParentOffset) {
    ::android::status_t status =
        aParcel->mParcel.writeEmbeddedBuffer(aBuffer, aBufferSize, aBufferHandle,
                                             aParentBufferHandle, aParentOffset);
    return status != ::android::OK;
}

int
parcel_read_embedded_buffer(Parcel* aParcel,
                          size_t aBufferSize, size_t* aBufferHandle,
                          size_t aParentBufferHandle, size_t aParentOffset,
                          char *aBuffer) {
    const void *out;
    ::android::status_t status =
        aParcel->mParcel.readEmbeddedBuffer(aBufferSize, aBufferHandle,
                                            aParentBufferHandle, aParentOffset,
                                            &out);
    if (status != ::android::OK) {
        return 1;
    }
    memcpy(aBuffer, out, aBufferSize);
    return 0;
}

size_t
parcel_data_size(Parcel* aParcel) {
    return aParcel->mParcel.dataSize();
}

void
parcel_set_data_position(Parcel* aParcel, size_t aPos) {
    aParcel->mParcel.setDataPosition(aPos);
}

int
parcel_write_native_handle_no_dup(Parcel* aParcel,
                              const native_handle_t *aHandle,
                              bool aEmbedded,
                              size_t aParentBufferHandle,
                              size_t aParentOffset) {
    ::android::status_t status =
        aParcel->mParcel.writeNativeHandleNoDup(aHandle, aEmbedded,
                                                aParentBufferHandle,
                                                aParentOffset);
    return status != ::android::OK;
}

int
parcel_read_nullable_native_handle_no_dup(Parcel* aParcel,
                                     const native_handle_t **aHandle,
                                     bool aEmbedded,
                                     size_t aParentBufferHandle,
                                     size_t aParentOffset) {
    ::android::status_t status;
    if (aEmbedded) {
        status =
            aParcel->mParcel.readEmbeddedNativeHandle(aParentBufferHandle,
                                                      aParentOffset,
                                                      aHandle);
    } else {
        status =
            aParcel->mParcel.readNativeHandleNoDup(aHandle);
    }
    return status != ::android::OK;
}


__DECL_C_END

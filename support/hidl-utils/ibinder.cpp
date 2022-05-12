/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
/* vim: set ts=8 sts=4 et sw=4 tw=80: */
#include <stdio.h>
#include <string.h>
#include <stdio.h>
#include <stddef.h>
#include <stdint.h>

#include <android/hidl/manager/1.0/IServiceManager.h>
#include <hidl/HidlBinderSupport.h>
#include <hidl/HidlTransportSupport.h>
#include <hidl/HidlTransportUtils.h>
#include <hidl/ServiceManagement.h>

#define IBAR_DESC "kaios.test.bar@1.0::IBar"
#define SERVICE_NAME "default"

using ::android::sp;
using ::android::hardware::Return;
using Transport = ::android::hidl::manager::V1_0::IServiceManager::Transport;

struct IBinderWrapper {
    int dummy;
    sp<::android::hardware::IBinder> ibinder;
    sp<::android::hardware::hidl_death_recipient> iDeath;
    sp<::android::hidl::base::V1_0::IBase> iBase;
};

#define __DECL_C_START extern "C" {
#define __DECL_C_END }


struct Parcel {
    int mDummy;
    ::android::hardware::Parcel mParcel;
};

__DECL_C_START

IBinderWrapper*
ibinder_query_ibinder(const char*aIface, size_t aIface_size,
                     const char* aServiceName, size_t aServiceName_size) {
    char* iface = new char[aIface_size + 1];
    memcpy(iface, aIface, aIface_size);
    iface[aIface_size] = 0;

    char* service_name = new char[aServiceName_size + 1];
    memcpy(service_name, aServiceName, aServiceName_size);
    service_name[aServiceName_size] = 0;

    const sp<::android::hidl::manager::V1_0::IServiceManager> sm =
        ::android::hardware::defaultServiceManager();
    if (sm == nullptr) {
        fprintf(stderr, "defaultServiceManager() is null\n");
        abort();
    }

    Return<Transport> transportRet = sm->getTransport(iface, service_name);
    if (!transportRet.isOk()) {
        fprintf(stderr, "getTransport() returns %s\n", transportRet.description().c_str());
        return nullptr;
    }
    Transport transport = transportRet;
    bool isHwbinder = transport == Transport::HWBINDER;
    bool isPass = transport == Transport::PASSTHROUGH;
    if (!isHwbinder && !isPass) {
        fprintf(stderr, "%s should use hwbinder or passthrough, but it is not! %d\n",
                iface, (int)transport);
    }

    sp<::android::hardware::IBinder> binder;
    sp<::android::hidl::base::V1_0::IBase> base;
    if (isHwbinder) {
        Return<sp<::android::hidl::base::V1_0::IBase>> ret =
            sm->get(iface, service_name);
        if (!ret.isOk()) {
            fprintf(stderr, "%s@%s is not found: %s\n", iface, service_name,
                    ret.description().c_str());
            return nullptr;
        }
        base = ret;
        if (base == nullptr) {
            fprintf(stderr, "%s@%s is not found\n", iface, service_name);
            return nullptr;
        }
        Return<bool> canCastRet =
            ::android::hardware::details::canCastInterface(base.get(), iface, false);
        if (!canCastRet.isOk()) {
            fprintf(stderr, "Can not cast to %s\n", iface);
            return nullptr;
        }

        binder = ::android::hardware::toBinder(base);
    } else {
        const sp<::android::hidl::manager::V1_0::IServiceManager> pm =
            ::android::hardware::getPassthroughServiceManager();
        if (pm == nullptr) {
            fprintf(stderr, "getPassthroughServiceManager() returns nullptr\n");
            return nullptr;
        }
        Return<sp<::android::hidl::base::V1_0::IBase>> ret =
            pm->get(iface, service_name);
        if (!ret.isOk()) {
            fprintf(stderr, "%s@%s is not found: %s\n", iface, service_name,
                    ret.description().c_str());
            return nullptr;
        }
        base = ret;
        if (base == nullptr) {
            fprintf(stderr, "%s@%s is not found\n", iface, service_name);
            return nullptr;
        }
        Return<bool> canCastRet =
            ::android::hardware::details::canCastInterface(base.get(), iface, false);
        if (!canCastRet.isOk()) {
            fprintf(stderr, "Can not cast to %s\n", iface);
            return nullptr;
        }

        binder = ::android::hardware::toBinder(base);
    }

    delete [] iface;
    delete [] service_name;

    if (binder == nullptr) {
        return nullptr;
    }

    auto wrapper = new IBinderWrapper;

    class DeathRecipient : public ::android::hardware::hidl_death_recipient {
     public:
      DeathRecipient(IBinderWrapper* aWrapper) : mWrapper(aWrapper) {
      }

      ~DeathRecipient() {
      }

      void serviceDied(uint64_t cookie, const android::wp<::android::hidl::base::V1_0::IBase>& who) override {
        if (mWrapper != nullptr) {
            mWrapper->iBase = nullptr;
        }
      }
     private:
      IBinderWrapper* mWrapper;
    };

    android::sp<::android::hardware::hidl_death_recipient> deathRecipient = new DeathRecipient(wrapper);
    base->linkToDeath(deathRecipient, 0);

    wrapper->ibinder = binder;
    wrapper->iDeath = deathRecipient;
    wrapper->iBase = base;
    return wrapper;
}

void
ibinder_delete(IBinderWrapper* aWrapper) {
    delete aWrapper;
}

int
ibinder_transact(IBinderWrapper* aWrapper, uint32_t aCode,
                 const Parcel* aData, Parcel* aReply,
                 uint32_t aFlags) {
    auto status = aWrapper->ibinder->transact(aCode, aData->mParcel,
                                              &aReply->mParcel, aFlags);
    return status != ::android::OK;
}

bool
ibinder_isalive(IBinderWrapper* aWrapper) {
    return aWrapper != nullptr && aWrapper->iBase != nullptr;
}

__DECL_C_END

#include "klyron_jsc.h"
#include "cpp/impl/internal.h"
#include <cstring>
#include <cstdlib>
#include <ctime>
#include <vector>

#ifdef _WIN32
#include <windows.h>
#include <wincrypt.h>
#else
#include <fcntl.h>
#include <unistd.h>
#endif

static bool random_bytes_internal(unsigned char* buf, size_t len) {
#ifdef _WIN32
    HCRYPTPROV prov = 0;
    if (!CryptAcquireContextW(&prov, nullptr, nullptr, PROV_RSA_FULL, CRYPT_VERIFYCONTEXT))
        return false;
    BOOL ok = CryptGenRandom(prov, (DWORD)len, buf);
    CryptReleaseContext(prov, 0);
    return ok != FALSE;
#else
    int fd = open("/dev/urandom", O_RDONLY);
    if (fd < 0) return false;
    size_t pos = 0;
    while (pos < len) {
        ssize_t r = read(fd, buf + pos, len - pos);
        if (r <= 0) { close(fd); return false; }
        pos += (size_t)r;
    }
    close(fd);
    return true;
#endif
}

klyron_jsc_value_t* klyron_jsc_random_fill(klyron_jsc_engine_t* engine, klyron_jsc_value_t* buf, size_t offset, size_t size) {
    klyron_jsc_value_t* result = nullptr;
    if (!engine || !buf) return nullptr;
    if (!JSValueIsObject(engine->ctx, buf->value)) {
        jsc_set_error(engine, "random_fill: argument is not an object");
        return nullptr;
    }
    JSObjectRef obj = (JSObjectRef)buf->value;
    JSValueRef exc = nullptr;
    JSTypedArrayType ta = JSValueGetTypedArrayType(engine->ctx, obj, &exc);
    if (exc) { jsc_capture_exception(engine, exc); return nullptr; }
    void* ptr = nullptr;
    size_t len = 0;
    if (ta != kJSTypedArrayTypeNone) {
        len = JSObjectGetTypedArrayLength(engine->ctx, obj, &exc);
        if (exc) { jsc_capture_exception(engine, exc); return nullptr; }
        size_t element_size = 1;
        switch (ta) {
            case kJSTypedArrayTypeInt16Array: case kJSTypedArrayTypeUint16Array: element_size = 2; break;
            case kJSTypedArrayTypeInt32Array: case kJSTypedArrayTypeUint32Array: case kJSTypedArrayTypeFloat32Array: element_size = 4; break;
            case kJSTypedArrayTypeFloat64Array: case kJSTypedArrayTypeBigInt64Array: case kJSTypedArrayTypeBigUint64Array: element_size = 8; break;
            default: element_size = 1; break;
        }
        size_t byte_len = len * element_size;
        JSObjectRef ab = JSObjectGetTypedArrayBuffer(engine->ctx, obj, &exc);
        if (exc || !ab) { jsc_capture_exception(engine, exc); return nullptr; }
        ptr = JSObjectGetArrayBufferBytesPtr(engine->ctx, ab, &exc);
        if (exc) { jsc_capture_exception(engine, exc); return nullptr; }
        size_t fill_size = size;
        if (fill_size == 0) fill_size = byte_len;
        fill_size = std::min(fill_size, byte_len - std::min(offset, byte_len));
        if (ptr) random_bytes_internal((unsigned char*)ptr + offset, fill_size);
    } else {
        size_t byte_len = JSObjectGetArrayBufferByteLength(engine->ctx, obj, &exc);
        if (exc || byte_len == 0) { jsc_capture_exception(engine, exc); return nullptr; }
        ptr = JSObjectGetArrayBufferBytesPtr(engine->ctx, obj, &exc);
        if (exc) { jsc_capture_exception(engine, exc); return nullptr; }
        size_t fill_size = size;
        if (fill_size == 0) fill_size = byte_len;
        fill_size = std::min(fill_size, byte_len - std::min(offset, byte_len));
        if (ptr) random_bytes_internal((unsigned char*)ptr + offset, fill_size);
    }
    return buf;
}

klyron_jsc_value_t* klyron_jsc_random_bytes(klyron_jsc_engine_t* engine, size_t length) {
    if (!engine || length == 0) return nullptr;
    std::vector<unsigned char> buf(length);
    if (!random_bytes_internal(buf.data(), length)) {
        jsc_set_error(engine, "random_bytes: failed to get random data");
        return nullptr;
    }
    void* bytes = std::malloc(length);
    if (!bytes) return nullptr;
    std::memcpy(bytes, buf.data(), length);
    JSValueRef exc = nullptr;
    JSValueRef ab = JSObjectMakeArrayBufferWithBytesNoCopy(
        engine->ctx, bytes, length,
        [](void* ptr, void* ctx) { std::free(ptr); },
        nullptr, &exc);
    if (exc || !ab) {
        std::free(bytes);
        jsc_capture_exception(engine, exc);
        return nullptr;
    }
    auto v = new klyron_jsc_value_t(engine->ctx, ab);
    v->protect();
    return v;
}

klyron_jsc_value_t* klyron_jsc_random_uuid(klyron_jsc_engine_t* engine) {
    if (!engine) return nullptr;
    unsigned char bytes[16];
    if (!random_bytes_internal(bytes, 16)) {
        jsc_set_error(engine, "random_uuid: failed to get random data");
        return nullptr;
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    char uuid[37];
    std::snprintf(uuid, sizeof(uuid),
        "%02x%02x%02x%02x-%02x%02x-%02x%02x-%02x%02x-%02x%02x%02x%02x%02x%02x",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11],
        bytes[12], bytes[13], bytes[14], bytes[15]);
    JSStringRef jsstr = jsc_string_from_cstr(uuid);
    if (!jsstr) return nullptr;
    JSValueRef val = JSValueMakeString(engine->ctx, jsstr);
    JSStringRelease(jsstr);
    auto v = new klyron_jsc_value_t(engine->ctx, val);
    v->protect();
    return v;
}

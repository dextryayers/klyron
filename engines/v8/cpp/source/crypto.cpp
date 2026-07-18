#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>
#include <cstdlib>
#include <algorithm>
#include <sstream>
#include <iomanip>

#if defined(_WIN32)
#include <windows.h>
#include <wincrypt.h>
#else
#include <fcntl.h>
#include <unistd.h>
#endif

static bool random_bytes_internal(unsigned char* buf, size_t size) {
#if defined(_WIN32)
    HCRYPTPROV hProv = 0;
    if (!CryptAcquireContext(&hProv, NULL, NULL, PROV_RSA_FULL, CRYPT_VERIFYCONTEXT))
        return false;
    bool ok = CryptGenRandom(hProv, (DWORD)size, buf) != 0;
    CryptReleaseContext(hProv, 0);
    return ok;
#else
    int fd = open("/dev/urandom", O_RDONLY);
    if (fd < 0) return false;
    size_t pos = 0;
    while (pos < size) {
        ssize_t r = read(fd, buf + pos, size - pos);
        if (r <= 0) { close(fd); return false; }
        pos += r;
    }
    close(fd);
    return true;
#endif
}

klyron_v8_value_t* klyron_v8_crypto_random_bytes(klyron_v8_context_t* ctx, size_t size) {
    if (!ctx || size == 0) return nullptr;

    auto* buf = static_cast<unsigned char*>(std::malloc(size));
    if (!buf) return nullptr;

    if (!random_bytes_internal(buf, size)) {
        std::free(buf);
        return nullptr;
    }

    auto iso = get_iso(ctx);
    v8::Locker locker(iso);
    v8::Isolate::Scope iso_scope(iso);
    v8::HandleScope scope(iso);
    auto context = get_ctx(ctx);
    v8::Context::Scope ctx_scope(context);

    auto ab = v8::ArrayBuffer::New(iso, size);
    std::memcpy(ab->GetBackingStore()->Data(), buf, size);
    std::free(buf);

    auto ta = v8::Uint8Array::New(ab, 0, size);
    return new klyron_v8_value(iso, ta, ctx->parent);
}

klyron_v8_string_result_t klyron_v8_crypto_random_uuid(klyron_v8_context_t* ctx) {
    klyron_v8_string_result_t result = {false, nullptr, 0, {0}};
    unsigned char bytes[16];
    if (!random_bytes_internal(bytes, 16)) {
        set_error(&result, "random bytes failed");
        return result;
    }

    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    std::ostringstream oss;
    oss << std::hex << std::setfill('0');
    for (int i = 0; i < 16; i++) {
        if (i == 4 || i == 6 || i == 8 || i == 10) oss << '-';
        oss << std::setw(2) << static_cast<int>(bytes[i]);
    }
    set_result(&result, oss.str());
    return result;
}

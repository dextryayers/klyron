#include "klyron_v8.h"
#include "cpp/impl/internal.h"

const char* klyron_v8_version(void) {
    return v8::V8::GetVersion();
}

int klyron_v8_major_version(void) {
    auto ver = v8::V8::GetVersion();
    return ver[0] - '0';
}

int klyron_v8_minor_version(void) {
    auto ver = v8::V8::GetVersion();
    return ver[2] - '0';
}

int klyron_v8_build_version(void) {
    auto ver = v8::V8::GetVersion();
    return ver[4] - '0';
}

int klyron_v8_patch_version(void) {
    return 0;
}

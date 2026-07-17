#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

const char* klyron_jsc_version(void) {
    return "JavaScriptCore (JSC) via C API";
}

void klyron_jsc_free_string(char* s) {
    std::free(s);
}

void klyron_jsc_free_buffer(unsigned char* buf) {
    std::free(buf);
}

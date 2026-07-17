#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstdlib>
#include <cstring>
#include <string>

void klyron_v8_set_string_result(klyron_v8_string_result_t* result,
                                 const char* str) {
    if (!result) return;
    if (str) {
        set_result(result, std::string(str));
    } else {
        set_result(result, "");
    }
}

void klyron_v8_free_string(char* str) {
    std::free(str);
}

void klyron_v8_free_buffer(unsigned char* buf) {
    std::free(buf);
}

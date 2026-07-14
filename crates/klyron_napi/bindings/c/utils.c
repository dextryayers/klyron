#include "utils.h"
#include <stdlib.h>

void klyron_napi_free_strings(char** strings, size_t count) {
    if (strings) {
        for (size_t i = 0; i < count; i++) free(strings[i]);
        free(strings);
    }
}

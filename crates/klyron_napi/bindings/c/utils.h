#ifndef KLYRON_NAPI_UTILS_H
#define KLYRON_NAPI_UTILS_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

void klyron_napi_free_strings(char** strings, size_t count);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_NAPI_UTILS_H */

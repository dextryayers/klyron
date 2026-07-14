#ifndef KLYRON_NAPI_TYPES_H
#define KLYRON_NAPI_TYPES_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    char* name;
    char* exports;
    uint32_t napi_version;
} klyron_napi_module_t;

typedef struct {
    char** paths;
    size_t path_count;
    bool cache_enabled;
} klyron_napi_config_t;

typedef struct {
    uint32_t major;
    uint32_t minor;
    uint32_t patch;
} klyron_napi_version_t;

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_NAPI_TYPES_H */

#ifndef KLYRON_NAPI_H
#define KLYRON_NAPI_H

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

typedef struct klyron_napi_loader_t klyron_napi_loader_t;

klyron_napi_loader_t* klyron_napi_loader_new(void);
klyron_napi_loader_t* klyron_napi_loader_with_config(const char* config_json);
void klyron_napi_loader_free(klyron_napi_loader_t* loader);

klyron_napi_module_t* klyron_napi_load(klyron_napi_loader_t* loader, const char* name);
void klyron_napi_module_free(klyron_napi_module_t* module);

char** klyron_napi_list_loaded(klyron_napi_loader_t* loader, size_t* count);
bool klyron_napi_is_loaded(klyron_napi_loader_t* loader, const char* name);
bool klyron_napi_unload(klyron_napi_loader_t* loader, const char* name);
void klyron_napi_clear(klyron_napi_loader_t* loader);
uint32_t klyron_napi_version(klyron_napi_loader_t* loader);
bool klyron_napi_is_napi_module(const char* name);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_NAPI_H */

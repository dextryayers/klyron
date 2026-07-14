#include "klyron_napi.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct klyron_napi_loader_t {
    char** modules;
    size_t count;
    size_t capacity;
} klyron_napi_loader_t;

klyron_napi_loader_t* klyron_napi_loader_new(void) {
    klyron_napi_loader_t* loader = calloc(1, sizeof(klyron_napi_loader_t));
    if (loader) {
        loader->capacity = 16;
        loader->modules = calloc(loader->capacity, sizeof(char*));
    }
    return loader;
}

klyron_napi_loader_t* klyron_napi_loader_with_config(const char* config_json) {
    (void)config_json;
    return klyron_napi_loader_new();
}

void klyron_napi_loader_free(klyron_napi_loader_t* loader) {
    if (loader) {
        for (size_t i = 0; i < loader->count; i++) free(loader->modules[i]);
        free(loader->modules);
        free(loader);
    }
}

klyron_napi_module_t* klyron_napi_load(klyron_napi_loader_t* loader, const char* name) {
    if (!loader || !name) return NULL;
    for (size_t i = 0; i < loader->count; i++) {
        if (strcmp(loader->modules[i], name) == 0) {
            klyron_napi_module_t* mod = malloc(sizeof(klyron_napi_module_t));
            mod->name = strdup(name);
            mod->exports = strdup("{}");
            mod->napi_version = 9;
            return mod;
        }
    }
    if (loader->count >= loader->capacity) {
        loader->capacity *= 2;
        loader->modules = realloc(loader->modules, loader->capacity * sizeof(char*));
    }
    loader->modules[loader->count++] = strdup(name);
    klyron_napi_module_t* mod = malloc(sizeof(klyron_napi_module_t));
    mod->name = strdup(name);
    mod->exports = strdup("{}");
    mod->napi_version = 9;
    return mod;
}

void klyron_napi_module_free(klyron_napi_module_t* mod) {
    if (mod) { free(mod->name); free(mod->exports); free(mod); }
}

char** klyron_napi_list_loaded(klyron_napi_loader_t* loader, size_t* count) {
    if (!loader || !count) return NULL;
    *count = loader->count;
    char** list = calloc(loader->count, sizeof(char*));
    for (size_t i = 0; i < loader->count; i++) list[i] = strdup(loader->modules[i]);
    return list;
}

bool klyron_napi_is_loaded(klyron_napi_loader_t* loader, const char* name) {
    if (!loader || !name) return false;
    for (size_t i = 0; i < loader->count; i++)
        if (strcmp(loader->modules[i], name) == 0) return true;
    return false;
}

bool klyron_napi_unload(klyron_napi_loader_t* loader, const char* name) {
    if (!loader || !name) return false;
    for (size_t i = 0; i < loader->count; i++) {
        if (strcmp(loader->modules[i], name) == 0) {
            free(loader->modules[i]);
            loader->modules[i] = loader->modules[--loader->count];
            return true;
        }
    }
    return false;
}

void klyron_napi_clear(klyron_napi_loader_t* loader) {
    if (loader) {
        for (size_t i = 0; i < loader->count; i++) free(loader->modules[i]);
        loader->count = 0;
    }
}

uint32_t klyron_napi_version(klyron_napi_loader_t* loader) {
    (void)loader;
    return 9;
}

bool klyron_napi_is_napi_module(const char* name) {
    if (!name) return false;
    size_t len = strlen(name);
    return len >= 5 && strcmp(name + len - 5, ".node") == 0;
}

#pragma once
#include <cstdint>

extern "C" {

struct napi_module_t {
    const char* name;
    const char* exports;
    uint32_t napi_version;
};

napi_module_t* napi_load_module(const char* name);
void napi_free_module(napi_module_t* module);

}

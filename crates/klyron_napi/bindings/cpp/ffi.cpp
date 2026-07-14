#include "ffi.hpp"
#include "api.hpp"
#include <cstring>
#include <memory>

extern "C" {

napi_module_t* napi_load_module(const char* name) {
    if (!name) return nullptr;
    auto loader = std::make_unique<klyron_napi::NapiLoader>();
    try {
        auto mod = loader->load(name);
        auto c_name = strdup(mod->name.c_str());
        auto c_exports = strdup("{}");
        auto result = new napi_module_t{c_name, c_exports, 9};
        return result;
    } catch (...) {
        return nullptr;
    }
}

void napi_free_module(napi_module_t* module) {
    if (module) {
        free(const_cast<char*>(module->name));
        free(const_cast<char*>(module->exports));
        delete module;
    }
}

}

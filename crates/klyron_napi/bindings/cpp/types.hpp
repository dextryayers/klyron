#pragma once
#include <string>
#include <unordered_map>
#include <vector>
#include <any>

namespace klyron_napi {

struct NapiModule {
    std::string name;
    std::unordered_map<std::string, std::any> exports;
};

struct NapiLoaderConfig {
    std::vector<std::string> module_paths;
    bool cache_enabled = true;
};

struct NapiVersion {
    uint32_t major = 9;
    uint32_t minor = 0;
    uint32_t patch = 0;
};

} // namespace klyron_napi

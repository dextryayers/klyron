#pragma once
#include "types.hpp"
#include "api.hpp"
#include <string>

namespace klyron_napi {

class NapiLoaderBuilder {
public:
    NapiLoaderBuilder();
    NapiLoaderBuilder& module_path(const std::string& path);
    NapiLoaderBuilder& cache_enabled(bool enabled);
    NapiLoader build();

private:
    NapiLoaderConfig config_;
};

} // namespace klyron_napi

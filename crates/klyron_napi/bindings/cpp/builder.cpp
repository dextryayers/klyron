#include "builder.hpp"

namespace klyron_napi {

NapiLoaderBuilder::NapiLoaderBuilder() : config_() {}

NapiLoaderBuilder& NapiLoaderBuilder::module_path(const std::string& path) {
    config_.module_paths.push_back(path);
    return *this;
}

NapiLoaderBuilder& NapiLoaderBuilder::cache_enabled(bool enabled) {
    config_.cache_enabled = enabled;
    return *this;
}

NapiLoader NapiLoaderBuilder::build() {
    return NapiLoader(config_);
}

} // namespace klyron_napi

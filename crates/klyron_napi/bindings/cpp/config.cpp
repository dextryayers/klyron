#include "config.hpp"

namespace klyron_napi {

NapiConfig::NapiConfig() : loader_config_(), napi_version_(9) {}

NapiConfig::NapiConfig(const NapiLoaderConfig& loader_config)
    : loader_config_(loader_config), napi_version_(9) {}

NapiConfig NapiConfig::with_loader(const NapiLoaderConfig& loader_config) const {
    return NapiConfig(loader_config);
}

NapiLoaderConfig NapiConfig::loader_config() const { return loader_config_; }

uint32_t NapiConfig::napi_version() const { return napi_version_; }

NapiConfig NapiConfig::defaults() { return NapiConfig(); }

} // namespace klyron_napi

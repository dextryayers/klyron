#pragma once
#include "types.hpp"

namespace klyron_napi {

class NapiConfig {
public:
    NapiConfig();
    explicit NapiConfig(const NapiLoaderConfig& loader_config);
    NapiConfig with_loader(const NapiLoaderConfig& loader_config) const;
    NapiLoaderConfig loader_config() const;
    uint32_t napi_version() const;
    static NapiConfig defaults();

private:
    NapiLoaderConfig loader_config_;
    uint32_t napi_version_ = 9;
};

} // namespace klyron_napi

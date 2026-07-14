#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_napi {

class NapiLoader {
public:
    NapiLoader();
    explicit NapiLoader(const NapiLoaderConfig& config);

    std::shared_ptr<NapiModule> load(const std::string& name);
    std::vector<std::string> list_loaded() const;
    bool is_loaded(const std::string& name) const;
    bool unload(const std::string& name);
    void clear();
    size_t symbol_count() const;
    uint32_t napi_version() const;
    static bool is_napi_module(const std::string& name);

private:
    std::unordered_map<std::string, std::shared_ptr<NapiModule>> loaded_modules_;
    NapiLoaderConfig config_;
};

class NapiClient {
public:
    NapiClient();
    std::shared_ptr<NapiModule> load(const std::string& name);
    std::vector<std::string> list() const;
    bool unload(const std::string& name);
    void clear();
    uint32_t version() const;

private:
    NapiLoader loader_;
};

} // namespace klyron_napi

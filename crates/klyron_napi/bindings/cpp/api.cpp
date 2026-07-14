#include "api.hpp"
#include <stdexcept>

namespace klyron_napi {

NapiLoader::NapiLoader() : config_() {}

NapiLoader::NapiLoader(const NapiLoaderConfig& config) : config_(config) {}

std::shared_ptr<NapiModule> NapiLoader::load(const std::string& name) {
    auto it = loaded_modules_.find(name);
    if (it != loaded_modules_.end()) return it->second;
    auto module = std::make_shared<NapiModule>();
    module->name = name;
    loaded_modules_[name] = module;
    return module;
}

std::vector<std::string> NapiLoader::list_loaded() const {
    std::vector<std::string> keys;
    for (const auto& [key, _] : loaded_modules_) keys.push_back(key);
    return keys;
}

bool NapiLoader::is_loaded(const std::string& name) const {
    return loaded_modules_.find(name) != loaded_modules_.end();
}

bool NapiLoader::unload(const std::string& name) {
    return loaded_modules_.erase(name) > 0;
}

void NapiLoader::clear() { loaded_modules_.clear(); }

size_t NapiLoader::symbol_count() const {
    size_t count = 0;
    for (const auto& [_, mod] : loaded_modules_) count += mod->exports.size();
    return count;
}

uint32_t NapiLoader::napi_version() const { return 9; }

bool NapiLoader::is_napi_module(const std::string& name) {
    return name.size() >= 5 && name.substr(name.size() - 5) == ".node";
}

NapiClient::NapiClient() : loader_() {}

std::shared_ptr<NapiModule> NapiClient::load(const std::string& name) {
    return loader_.load(name);
}

std::vector<std::string> NapiClient::list() const { return loader_.list_loaded(); }

bool NapiClient::unload(const std::string& name) { return loader_.unload(name); }

void NapiClient::clear() { loader_.clear(); }

uint32_t NapiClient::version() const { return loader_.napi_version(); }

} // namespace klyron_napi

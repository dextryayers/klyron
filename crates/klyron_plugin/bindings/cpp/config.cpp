#include "config.hpp"

namespace klyron::_plugin {

PluginConfigBuilder& PluginConfigBuilder::with_version(const std::string& v) {
    config_.version = v;
    return *this;
}

PluginConfig PluginConfigBuilder::build() {
    return config_;
}

}

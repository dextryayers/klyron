#include "builder.hpp"

namespace klyron::_plugin {

PluginBuilder::PluginBuilder() : config_{} {}

PluginBuilder& PluginBuilder::set_config(const PluginConfig& cfg) {
    config_ = cfg;
    return *this;
}

PluginConfig PluginBuilder::config() const {
    return config_;
}

}

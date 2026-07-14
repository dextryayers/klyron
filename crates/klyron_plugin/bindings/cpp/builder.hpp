#pragma once
#include "types.hpp"
#include "config.hpp"

namespace klyron::_plugin {

class PluginBuilder {
public:
    PluginBuilder();
    PluginBuilder& set_config(const PluginConfig& cfg);
    PluginConfig config() const;
private:
    PluginConfig config_;
};

}

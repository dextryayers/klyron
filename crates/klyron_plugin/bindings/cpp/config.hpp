#pragma once
#include "types.hpp"

namespace klyron::_plugin {

class PluginConfigBuilder {
public:
    PluginConfigBuilder& with_version(const std::string& v);
    PluginConfig build();
private:
    PluginConfig config_;
};

}

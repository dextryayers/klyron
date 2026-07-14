#pragma once
#include "types.hpp"

namespace klyron::_plugin {

class PluginClient {
public:
    explicit PluginClient(const PluginConfig& config);
    void execute();
private:
    PluginConfig config_;
};

}

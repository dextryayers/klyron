#pragma once
#include "types.hpp"

namespace klyron::_plugin {

class PluginApi {
public:
    PluginApi();
    void execute();
    static std::string version();
};

}

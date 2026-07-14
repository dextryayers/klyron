#include "api.hpp"

namespace klyron::_plugin {

PluginApi::PluginApi() {}

void PluginApi::execute() {
}

std::string PluginApi::version() {
    return "klyron_plugin@0.1.0";
}

}

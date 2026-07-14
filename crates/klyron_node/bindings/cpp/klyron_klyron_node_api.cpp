#include "klyron_klyron_node_api.hpp"
#include <string>

namespace klyron {

NodeGlobalsApi::NodeGlobalsApi() {}

std::string NodeGlobalsApi::version() const {
    return "klyron_node 0.1.0";
}

bool NodeGlobalsApi::ping() {
    return true;
}

}

#include "builder.hpp"

namespace klyron::_deploy {

DeployBuilder::DeployBuilder() : config_{} {}

DeployBuilder& DeployBuilder::set_config(const DeployConfig& cfg) {
    config_ = cfg;
    return *this;
}

DeployConfig DeployBuilder::config() const {
    return config_;
}

}

#include "config.hpp"

namespace klyron::_deploy {

DeployConfigBuilder& DeployConfigBuilder::with_version(const std::string& v) {
    config_.version = v;
    return *this;
}

DeployConfig DeployConfigBuilder::build() {
    return config_;
}

}

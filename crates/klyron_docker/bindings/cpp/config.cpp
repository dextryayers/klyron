#include "config.hpp"

namespace klyron::_docker {

DockerConfigBuilder& DockerConfigBuilder::with_version(const std::string& v) {
    config_.version = v;
    return *this;
}

DockerConfig DockerConfigBuilder::build() {
    return config_;
}

}

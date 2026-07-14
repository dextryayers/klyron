#include "builder.hpp"

namespace klyron::_docker {

DockerBuilder::DockerBuilder() : config_{} {}

DockerBuilder& DockerBuilder::set_config(const DockerConfig& cfg) {
    config_ = cfg;
    return *this;
}

DockerConfig DockerBuilder::config() const {
    return config_;
}

}

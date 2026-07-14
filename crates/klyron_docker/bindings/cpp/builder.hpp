#pragma once
#include "types.hpp"
#include "config.hpp"

namespace klyron::_docker {

class DockerBuilder {
public:
    DockerBuilder();
    DockerBuilder& set_config(const DockerConfig& cfg);
    DockerConfig config() const;
private:
    DockerConfig config_;
};

}

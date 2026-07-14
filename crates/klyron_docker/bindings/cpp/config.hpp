#pragma once
#include "types.hpp"

namespace klyron::_docker {

class DockerConfigBuilder {
public:
    DockerConfigBuilder& with_version(const std::string& v);
    DockerConfig build();
private:
    DockerConfig config_;
};

}

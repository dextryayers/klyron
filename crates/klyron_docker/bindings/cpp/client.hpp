#pragma once
#include "types.hpp"

namespace klyron::_docker {

class DockerClient {
public:
    explicit DockerClient(const DockerConfig& config);
    void execute();
private:
    DockerConfig config_;
};

}

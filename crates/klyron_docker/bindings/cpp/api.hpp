#pragma once
#include "types.hpp"

namespace klyron::_docker {

class DockerApi {
public:
    DockerApi();
    void execute();
    static std::string version();
};

}

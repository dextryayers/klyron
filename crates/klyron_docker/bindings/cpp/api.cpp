#include "api.hpp"

namespace klyron::_docker {

DockerApi::DockerApi() {}

void DockerApi::execute() {
}

std::string DockerApi::version() {
    return "klyron_docker@0.1.0";
}

}

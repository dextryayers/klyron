#include "client.hpp"

namespace klyron::_docker {

DockerClient::DockerClient(const DockerConfig& config)
    : config_(config) {}

void DockerClient::execute() {
}

}

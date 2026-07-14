#include "ffi.hpp"
#include "types.hpp"

extern "C" {
    void* docker_create_config() {
        auto* cfg = new klyron::_docker::DockerConfig();
        return static_cast<void*>(cfg);
    }

    void docker_free_config(void* ptr) {
        delete static_cast<klyron::_docker::DockerConfig*>(ptr);
    }
}

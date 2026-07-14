#include "ffi.hpp"
#include "types.hpp"

extern "C" {
    void* deploy_create_config() {
        auto* cfg = new klyron::_deploy::DeployConfig();
        return static_cast<void*>(cfg);
    }

    void deploy_free_config(void* ptr) {
        delete static_cast<klyron::_deploy::DeployConfig*>(ptr);
    }
}

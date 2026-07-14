#include "ffi.hpp"
#include "types.hpp"

extern "C" {
    void* workspace_create_config() {
        auto* cfg = new klyron::_workspace::WorkspaceConfig();
        return static_cast<void*>(cfg);
    }

    void workspace_free_config(void* ptr) {
        delete static_cast<klyron::_workspace::WorkspaceConfig*>(ptr);
    }
}

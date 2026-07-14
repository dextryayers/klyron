#include "ffi.hpp"
#include "types.hpp"

extern "C" {
    void* shell_create_config() {
        auto* cfg = new klyron::_shell::ShellConfig();
        return static_cast<void*>(cfg);
    }

    void shell_free_config(void* ptr) {
        delete static_cast<klyron::_shell::ShellConfig*>(ptr);
    }
}

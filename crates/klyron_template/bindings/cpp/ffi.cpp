#include "ffi.hpp"
#include "types.hpp"

extern "C" {
    void* template_create_config() {
        auto* cfg = new klyron::_template::TemplateConfig();
        return static_cast<void*>(cfg);
    }

    void template_free_config(void* ptr) {
        delete static_cast<klyron::_template::TemplateConfig*>(ptr);
    }
}

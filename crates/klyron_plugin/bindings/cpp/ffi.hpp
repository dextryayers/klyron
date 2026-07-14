#pragma once
#include "types.hpp"

extern "C" {
    void* plugin_create_config();
    void plugin_free_config(void* ptr);
}

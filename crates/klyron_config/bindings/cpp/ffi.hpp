#pragma once
#include "types.hpp"

extern "C" {
    void* config_create_config();
    void config_free_config(void* ptr);
}

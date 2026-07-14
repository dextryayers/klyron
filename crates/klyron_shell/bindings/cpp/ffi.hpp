#pragma once
#include "types.hpp"

extern "C" {
    void* shell_create_config();
    void shell_free_config(void* ptr);
}

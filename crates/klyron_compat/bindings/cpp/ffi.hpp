#pragma once
#include "types.hpp"

extern "C" {
    void* compat_create_config();
    void compat_free_config(void* ptr);
}

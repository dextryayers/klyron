#pragma once
#include "types.hpp"

extern "C" {
    void* adapter_create_config();
    void adapter_free_config(void* ptr);
}

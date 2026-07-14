#pragma once
#include "types.hpp"

extern "C" {
    void* template_create_config();
    void template_free_config(void* ptr);
}

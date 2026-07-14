#pragma once
#include "types.hpp"

extern "C" {
    void* workspace_create_config();
    void workspace_free_config(void* ptr);
}

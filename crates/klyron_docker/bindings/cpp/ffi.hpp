#pragma once
#include "types.hpp"

extern "C" {
    void* docker_create_config();
    void docker_free_config(void* ptr);
}

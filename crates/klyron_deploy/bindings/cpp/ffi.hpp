#pragma once
#include "types.hpp"

extern "C" {
    void* deploy_create_config();
    void deploy_free_config(void* ptr);
}

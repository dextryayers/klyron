#pragma once
#include "types.hpp"

extern "C" {
    void* telemetry_create_config();
    void telemetry_free_config(void* ptr);
}

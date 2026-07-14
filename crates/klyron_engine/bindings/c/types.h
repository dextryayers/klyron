#ifndef KLYRON_ENGINE_BINDINGS_TYPES_H
#define KLYRON_ENGINE_BINDINGS_TYPES_H

#include <stdbool.h>
#include <stdint.h>

typedef struct klyron_engine_config_t {
  bool enabled;
} klyron_engine_config_t;

typedef struct klyron_engine_result_t {
  bool success;
  char* data;
  char* error;
} klyron_engine_result_t;

typedef enum klyron_engine_status_t {
  STATUS_ACTIVE,
  STATUS_INACTIVE,
  STATUS_ERROR
} klyron_engine_status_t;

#endif

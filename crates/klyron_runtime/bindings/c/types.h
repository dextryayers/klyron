#ifndef KLYRON_RUNTIME_BINDINGS_TYPES_H
#define KLYRON_RUNTIME_BINDINGS_TYPES_H

#include <stdbool.h>
#include <stdint.h>

typedef struct klyron_runtime_config_t {
  bool enabled;
} klyron_runtime_config_t;

typedef struct klyron_runtime_result_t {
  bool success;
  char* data;
  char* error;
} klyron_runtime_result_t;

typedef enum klyron_runtime_status_t {
  STATUS_ACTIVE,
  STATUS_INACTIVE,
  STATUS_ERROR
} klyron_runtime_status_t;

#endif

#ifndef KLYRON_UTILS_BINDINGS_TYPES_H
#define KLYRON_UTILS_BINDINGS_TYPES_H

#include <stdbool.h>
#include <stdint.h>

typedef struct klyron_utils_config_t {
  bool enabled;
} klyron_utils_config_t;

typedef struct klyron_utils_result_t {
  bool success;
  char* data;
  char* error;
} klyron_utils_result_t;

typedef enum klyron_utils_status_t {
  STATUS_ACTIVE,
  STATUS_INACTIVE,
  STATUS_ERROR
} klyron_utils_status_t;

#endif

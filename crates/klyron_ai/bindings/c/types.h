#ifndef KLYRON_AI_BINDINGS_TYPES_H
#define KLYRON_AI_BINDINGS_TYPES_H

#include <stdbool.h>
#include <stdint.h>

typedef struct klyron_ai_config_t {
  bool enabled;
} klyron_ai_config_t;

typedef struct klyron_ai_result_t {
  bool success;
  char* data;
  char* error;
} klyron_ai_result_t;

typedef enum klyron_ai_status_t {
  STATUS_ACTIVE,
  STATUS_INACTIVE,
  STATUS_ERROR
} klyron_ai_status_t;

#endif

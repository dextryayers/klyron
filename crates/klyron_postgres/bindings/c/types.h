#ifndef KLYRON_POSTGRES_BINDINGS_TYPES_H
#define KLYRON_POSTGRES_BINDINGS_TYPES_H

#include <stdbool.h>
#include <stdint.h>

typedef struct klyron_postgres_config_t {
  bool enabled;
} klyron_postgres_config_t;

typedef struct klyron_postgres_result_t {
  bool success;
  char* data;
  char* error;
} klyron_postgres_result_t;

typedef enum klyron_postgres_status_t {
  STATUS_ACTIVE,
  STATUS_INACTIVE,
  STATUS_ERROR
} klyron_postgres_status_t;

#endif

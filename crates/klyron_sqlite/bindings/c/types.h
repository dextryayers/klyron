#ifndef KLYRON_SQLITE_BINDINGS_TYPES_H
#define KLYRON_SQLITE_BINDINGS_TYPES_H

#include <stdbool.h>
#include <stdint.h>

typedef struct klyron_sqlite_config_t {
  bool enabled;
} klyron_sqlite_config_t;

typedef struct klyron_sqlite_result_t {
  bool success;
  char* data;
  char* error;
} klyron_sqlite_result_t;

typedef enum klyron_sqlite_status_t {
  STATUS_ACTIVE,
  STATUS_INACTIVE,
  STATUS_ERROR
} klyron_sqlite_status_t;

#endif

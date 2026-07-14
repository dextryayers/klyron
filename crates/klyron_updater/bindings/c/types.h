#ifndef KLYRON_UPDATER_BINDINGS_TYPES_H
#define KLYRON_UPDATER_BINDINGS_TYPES_H

#include <stdbool.h>
#include <stdint.h>

typedef struct klyron_updater_config_t {
  bool enabled;
} klyron_updater_config_t;

typedef struct klyron_updater_result_t {
  bool success;
  char* data;
  char* error;
} klyron_updater_result_t;

typedef enum klyron_updater_status_t {
  STATUS_ACTIVE,
  STATUS_INACTIVE,
  STATUS_ERROR
} klyron_updater_status_t;

#endif

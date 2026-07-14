#ifndef KLYRON_CLI_BINDINGS_TYPES_H
#define KLYRON_CLI_BINDINGS_TYPES_H

#include <stdbool.h>
#include <stdint.h>

typedef struct klyron_cli_config_t {
  bool enabled;
} klyron_cli_config_t;

typedef struct klyron_cli_result_t {
  bool success;
  char* data;
  char* error;
} klyron_cli_result_t;

typedef enum klyron_cli_status_t {
  STATUS_ACTIVE,
  STATUS_INACTIVE,
  STATUS_ERROR
} klyron_cli_status_t;

#endif

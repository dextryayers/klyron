#ifndef KLYRON_UPDATER_BINDINGS_ERRORS_H
#define KLYRON_UPDATER_BINDINGS_ERRORS_H

typedef enum klyron_updater_error_code_t {
  ERROR_NONE = 0,
  ERROR_NOT_FOUND,
  ERROR_INVALID_INPUT,
  ERROR_OPERATION_FAILED
} klyron_updater_error_code_t;

const char* klyron_updater_error_message(klyron_updater_error_code_t code);

#endif

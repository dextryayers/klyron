#ifndef KLYRON_UTILS_BINDINGS_ERRORS_H
#define KLYRON_UTILS_BINDINGS_ERRORS_H

typedef enum klyron_utils_error_code_t {
  ERROR_NONE = 0,
  ERROR_NOT_FOUND,
  ERROR_INVALID_INPUT,
  ERROR_OPERATION_FAILED
} klyron_utils_error_code_t;

const char* klyron_utils_error_message(klyron_utils_error_code_t code);

#endif

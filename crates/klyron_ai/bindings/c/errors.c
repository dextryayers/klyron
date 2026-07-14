#include "errors.h"

const char* klyron_ai_error_message(klyron_ai_error_code_t code) {
  switch (code) {
    case ERROR_NONE: return "ok";
    case ERROR_NOT_FOUND: return "not found";
    case ERROR_INVALID_INPUT: return "invalid input";
    case ERROR_OPERATION_FAILED: return "operation failed";
    default: return "unknown error";
  }
}

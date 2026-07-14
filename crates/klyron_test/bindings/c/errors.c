#include "errors.h"

const char* klyron_test_error_string(klyron_test_error_t err) {
    switch (err) {
        case KLYRON_TEST_OK: return "success";
        case KLYRON_TEST_ERR_FAILED: return "operation failed";
        default: return "unknown error";
    }
}

#include "errors.h"

const char* klyron_bench_error_string(klyron_bench_error_t err) {
    switch (err) {
        case KLYRON_BENCH_OK: return "success";
        case KLYRON_BENCH_ERR_FAILED: return "operation failed";
        default: return "unknown error";
    }
}

#ifndef KLYRON_BENCH_ERRORS_H
#define KLYRON_BENCH_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_BENCH_OK = 0,
    KLYRON_BENCH_ERR_FAILED = -1,
} klyron_bench_error_t;

const char* klyron_bench_error_string(klyron_bench_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_BENCH_ERRORS_H */

#ifndef KLYRON_TEST_ERRORS_H
#define KLYRON_TEST_ERRORS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    KLYRON_TEST_OK = 0,
    KLYRON_TEST_ERR_FAILED = -1,
} klyron_test_error_t;

const char* klyron_test_error_string(klyron_test_error_t err);

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_TEST_ERRORS_H */

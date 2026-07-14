#ifndef KLYRON_TEST_TYPES_H
#define KLYRON_TEST_TYPES_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool enabled;
    bool verbose;
} klyron_test_config_t;

#ifdef __cplusplus
}
#endif

#endif /* KLYRON_TEST_TYPES_H */

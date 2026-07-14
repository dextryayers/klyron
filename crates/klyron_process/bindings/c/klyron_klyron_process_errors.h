#ifndef KLYRON_KLYRON_PROCESS_ERRORS_H
#define KLYRON_KLYRON_PROCESS_ERRORS_H

typedef enum { KLYRON_KLYRON_PROCESS_OK = 0, KLYRON_KLYRON_PROCESS_ERR_GENERIC = -1 } klyron_klyron_process_error_t;
const char* klyron_klyron_process_error_string(klyron_klyron_process_error_t err);

#endif

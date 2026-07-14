#ifndef KLYRON_TYPES_H
#define KLYRON_TYPES_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

typedef struct {
    char *data;
    size_t len;
    size_t cap;
} klyron_string_t;

typedef struct {
    int32_t status;
    char *status_text;
    char *body;
    char **headers;
    size_t headers_len;
} klyron_response_t;

typedef struct {
    char *stdout_data;
    char *stderr_data;
    int exit_code;
    bool success;
} klyron_process_result_t;

typedef struct {
    char *path;
    uint64_t size;
    bool is_dir;
    bool is_file;
    int64_t modified;
} klyron_file_info_t;

typedef struct {
    klyron_file_info_t *entries;
    size_t count;
} klyron_dir_list_t;

void klyron_dir_list_free(klyron_dir_list_t *list);

#endif

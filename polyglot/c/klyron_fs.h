#ifndef KLYRON_FS_H
#define KLYRON_FS_H

#include "klyron_types.h"
#include <stdbool.h>

klyron_string_t klyron_fs_read_file(const char *path);
bool klyron_fs_write_file(const char *path, const char *data);
bool klyron_fs_exists(const char *path);
bool klyron_fs_mkdir(const char *path);
bool klyron_fs_remove(const char *path);
klyron_file_info_t klyron_fs_stat(const char *path);
bool klyron_fs_copy(const char *src, const char *dst);
klyron_dir_list_t klyron_fs_read_dir(const char *path);
bool klyron_fs_rename(const char *old, const char *new_);
int64_t klyron_fs_file_size(const char *path);
bool klyron_fs_is_dir(const char *path);
bool klyron_fs_is_file(const char *path);

#endif

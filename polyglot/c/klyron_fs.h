#ifndef KLYRON_FS_H
#define KLYRON_FS_H

#include "klyron_types.h"

char *klyron_fs_read_file(const char *path);
bool klyron_fs_write_file(const char *path, const char *data);
bool klyron_fs_exists(const char *path);
bool klyron_fs_mkdir(const char *path);

#endif

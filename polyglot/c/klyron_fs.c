#include "klyron_fs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>
#include <dirent.h>
#include <errno.h>

klyron_string_t klyron_fs_read_file(const char *path) {
    klyron_string_t result = {NULL, 0, 0};
    FILE *fp = fopen(path, "rb");
    if (!fp) return result;

    fseek(fp, 0, SEEK_END);
    long fsize = ftell(fp);
    fseek(fp, 0, SEEK_SET);
    if (fsize < 0) { fclose(fp); return result; }

    result.data = (char *)malloc((size_t)fsize + 1);
    if (!result.data) { fclose(fp); return result; }

    size_t nread = fread(result.data, 1, (size_t)fsize, fp);
    result.data[nread] = '\0';
    result.len = nread;
    result.cap = nread + 1;
    fclose(fp);
    return result;
}

bool klyron_fs_write_file(const char *path, const char *data) {
    FILE *fp = fopen(path, "wb");
    if (!fp) return false;
    size_t len = strlen(data);
    size_t written = fwrite(data, 1, len, fp);
    fclose(fp);
    return written == len;
}

bool klyron_fs_exists(const char *path) {
    return access(path, F_OK) == 0;
}

bool klyron_fs_mkdir(const char *path) {
    char tmp[1024];
    strncpy(tmp, path, sizeof(tmp) - 1);
    tmp[sizeof(tmp) - 1] = '\0';
    for (char *p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            mkdir(tmp, 0755);
            *p = '/';
        }
    }
    return mkdir(tmp, 0755) == 0 || errno == EEXIST;
}

bool klyron_fs_remove(const char *path) {
    return remove(path) == 0;
}

klyron_file_info_t klyron_fs_stat(const char *path) {
    klyron_file_info_t info = {NULL, 0, false, false, 0};
    struct stat st;
    if (stat(path, &st) != 0) return info;
    info.path = strdup(path);
    info.size = (uint64_t)st.st_size;
    info.is_dir = S_ISDIR(st.st_mode);
    info.is_file = S_ISREG(st.st_mode);
    info.modified = (int64_t)st.st_mtime;
    return info;
}

bool klyron_fs_copy(const char *src, const char *dst) {
    FILE *fsrc = fopen(src, "rb");
    if (!fsrc) return false;
    FILE *fdst = fopen(dst, "wb");
    if (!fdst) { fclose(fsrc); return false; }
    char buf[8192];
    size_t n;
    while ((n = fread(buf, 1, sizeof(buf), fsrc)) > 0) {
        fwrite(buf, 1, n, fdst);
    }
    fclose(fsrc);
    fclose(fdst);
    return true;
}

klyron_dir_list_t klyron_fs_read_dir(const char *path) {
    klyron_dir_list_t list = {NULL, 0};
    DIR *dir = opendir(path);
    if (!dir) return list;

    size_t cap = 32;
    list.entries = (klyron_file_info_t *)malloc(cap * sizeof(klyron_file_info_t));
    if (!list.entries) { closedir(dir); return list; }

    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0)
            continue;

        if (list.count >= cap) {
            cap *= 2;
            klyron_file_info_t *tmp = (klyron_file_info_t *)realloc(list.entries, cap * sizeof(klyron_file_info_t));
            if (!tmp) break;
            list.entries = tmp;
        }

        size_t full_len = strlen(path) + 1 + strlen(entry->d_name) + 1;
        char *full_path = (char *)malloc(full_len);
        snprintf(full_path, full_len, "%s/%s", path, entry->d_name);

        list.entries[list.count].path = full_path;
        struct stat st;
        if (stat(full_path, &st) == 0) {
            list.entries[list.count].size = (uint64_t)st.st_size;
            list.entries[list.count].is_dir = S_ISDIR(st.st_mode);
            list.entries[list.count].is_file = S_ISREG(st.st_mode);
            list.entries[list.count].modified = (int64_t)st.st_mtime;
        } else {
            list.entries[list.count].size = 0;
            list.entries[list.count].is_dir = (entry->d_type == DT_DIR);
            list.entries[list.count].is_file = (entry->d_type == DT_REG);
            list.entries[list.count].modified = 0;
        }
        list.count++;
    }
    closedir(dir);
    return list;
}

bool klyron_fs_rename(const char *old, const char *new_) {
    return rename(old, new_) == 0;
}

int64_t klyron_fs_file_size(const char *path) {
    struct stat st;
    if (stat(path, &st) != 0) return -1;
    return (int64_t)st.st_size;
}

bool klyron_fs_is_dir(const char *path) {
    struct stat st;
    if (stat(path, &st) != 0) return false;
    return S_ISDIR(st.st_mode);
}

bool klyron_fs_is_file(const char *path) {
    struct stat st;
    if (stat(path, &st) != 0) return false;
    return S_ISREG(st.st_mode);
}

void klyron_dir_list_free(klyron_dir_list_t *list) {
    if (list->entries) {
        for (size_t i = 0; i < list->count; i++) {
            free(list->entries[i].path);
        }
        free(list->entries);
        list->entries = NULL;
    }
    list->count = 0;
}

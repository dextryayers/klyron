#include "klyron_fs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

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
    FILE *fdst = fopen(dst, "wb");
    if (!fsrc || !fdst) {
        if (fsrc) fclose(fsrc);
        if (fdst) fclose(fdst);
        return false;
    }
    char buf[8192];
    size_t n;
    while ((n = fread(buf, 1, sizeof(buf), fsrc)) > 0) {
        fwrite(buf, 1, n, fdst);
    }
    fclose(fsrc);
    fclose(fdst);
    return true;
}

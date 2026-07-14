#include "klyron_fs.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <errno.h>

char *klyron_fs_read_file(const char *path)
{
    if (!path) return NULL;

    FILE *fp = fopen(path, "rb");
    if (!fp) return NULL;

    fseek(fp, 0, SEEK_END);
    long file_size = ftell(fp);
    rewind(fp);

    if (file_size < 0) {
        fclose(fp);
        return NULL;
    }

    char *data = malloc((size_t)file_size + 1);
    if (!data) {
        fclose(fp);
        return NULL;
    }

    size_t nread = fread(data, 1, (size_t)file_size, fp);
    fclose(fp);

    data[nread] = '\0';
    return data;
}

bool klyron_fs_write_file(const char *path, const char *data)
{
    if (!path || !data) return false;

    FILE *fp = fopen(path, "wb");
    if (!fp) return false;

    size_t len = strlen(data);
    size_t written = fwrite(data, 1, len, fp);
    fclose(fp);

    return written == len;
}

bool klyron_fs_exists(const char *path)
{
    if (!path) return false;
    struct stat st;
    return stat(path, &st) == 0;
}

bool klyron_fs_mkdir(const char *path)
{
    if (!path) return false;
    return mkdir(path, 0755) == 0 || errno == EEXIST;
}

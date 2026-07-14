#include "klyron_string.h"
#include <string.h>
#include <stdlib.h>

klyron_string_t klyron_string_new(const char *s)
{
    klyron_string_t result = {0};
    if (!s) return result;

    result.len = strlen(s);
    result.cap = result.len + 1;
    result.data = malloc(result.cap);
    if (result.data)
        memcpy(result.data, s, result.len + 1);
    return result;
}

void klyron_string_free(klyron_string_t *s)
{
    if (!s) return;
    free(s->data);
    s->data = NULL;
    s->len = 0;
    s->cap = 0;
}

klyron_string_t klyron_string_dup(const klyron_string_t *s)
{
    klyron_string_t result = {0};
    if (!s || !s->data) return result;

    result.len = s->len;
    result.cap = s->cap;
    result.data = malloc(result.cap);
    if (result.data)
        memcpy(result.data, s->data, result.len + 1);
    return result;
}

void klyron_string_append(klyron_string_t *dst, const char *src)
{
    if (!dst || !src) return;

    size_t src_len = strlen(src);
    size_t new_len = dst->len + src_len;

    if (new_len + 1 > dst->cap) {
        dst->cap = new_len + 1;
        char *tmp = realloc(dst->data, dst->cap);
        if (!tmp) return;
        dst->data = tmp;
    }

    memcpy(dst->data + dst->len, src, src_len + 1);
    dst->len = new_len;
}

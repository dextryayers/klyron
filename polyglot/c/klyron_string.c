#include "klyron_types.h"
#include <stdlib.h>
#include <string.h>

klyron_string_t klyron_string_new(const char *s) {
    klyron_string_t str;
    size_t len = s ? strlen(s) : 0;
    str.data = (char *)malloc(len + 1);
    if (str.data) {
        memcpy(str.data, s ? s : "", len + 1);
        str.len = len;
        str.cap = len + 1;
    } else {
        str.len = 0;
        str.cap = 0;
    }
    return str;
}

void klyron_string_free(klyron_string_t *s) {
    if (s->data) {
        free(s->data);
        s->data = NULL;
    }
    s->len = 0;
    s->cap = 0;
}

klyron_string_t klyron_string_dup(const klyron_string_t *s) {
    return klyron_string_new(s->data);
}

void klyron_string_append(klyron_string_t *s, const char *tail) {
    if (!tail) return;
    size_t tail_len = strlen(tail);
    size_t new_len = s->len + tail_len;
    if (new_len + 1 > s->cap) {
        s->cap = new_len + 1;
        s->data = (char *)realloc(s->data, s->cap);
    }
    memcpy(s->data + s->len, tail, tail_len + 1);
    s->len = new_len;
}

klyron_string_t klyron_string_substring(klyron_string_t *s, size_t start, size_t len) {
    klyron_string_t result = {NULL, 0, 0};
    if (!s->data || start >= s->len) return result;
    if (start + len > s->len) len = s->len - start;
    result.data = (char *)malloc(len + 1);
    if (!result.data) return result;
    memcpy(result.data, s->data + start, len);
    result.data[len] = '\0';
    result.len = len;
    result.cap = len + 1;
    return result;
}

void klyron_string_trim_left(klyron_string_t *s) {
    if (!s->data || s->len == 0) return;
    size_t start = 0;
    while (start < s->len && (s->data[start] == ' ' || s->data[start] == '\t' || s->data[start] == '\n' || s->data[start] == '\r'))
        start++;
    if (start > 0) {
        s->len -= start;
        memmove(s->data, s->data + start, s->len);
        s->data[s->len] = '\0';
    }
}

void klyron_string_trim_right(klyron_string_t *s) {
    if (!s->data || s->len == 0) return;
    while (s->len > 0 && (s->data[s->len - 1] == ' ' || s->data[s->len - 1] == '\t' || s->data[s->len - 1] == '\n' || s->data[s->len - 1] == '\r'))
        s->len--;
    s->data[s->len] = '\0';
}

klyron_string_t klyron_string_trim(klyron_string_t *s) {
    klyron_string_t r = klyron_string_dup(s);
    klyron_string_trim_left(&r);
    klyron_string_trim_right(&r);
    return r;
}

klyron_string_t *klyron_string_split(klyron_string_t *s, const char *delim, size_t *count) {
    *count = 0;
    if (!s->data || s->len == 0) return NULL;
    size_t delim_len = strlen(delim);
    if (delim_len == 0) return NULL;
    size_t cap = 8, n = 0;
    klyron_string_t *arr = (klyron_string_t *)malloc(cap * sizeof(klyron_string_t));
    if (!arr) return NULL;
    char *start = s->data;
    char *p;
    while ((p = strstr(start, delim)) != NULL) {
        size_t part_len = (size_t)(p - start);
        arr[n].data = (char *)malloc(part_len + 1);
        if (arr[n].data) {
            memcpy(arr[n].data, start, part_len);
            arr[n].data[part_len] = '\0';
            arr[n].len = part_len;
            arr[n].cap = part_len + 1;
        }
        n++;
        if (n >= cap) {
            cap *= 2;
            arr = (klyron_string_t *)realloc(arr, cap * sizeof(klyron_string_t));
        }
        start = p + delim_len;
    }
    size_t last_len = strlen(start);
    arr[n].data = (char *)malloc(last_len + 1);
    if (arr[n].data) {
        memcpy(arr[n].data, start, last_len + 1);
        arr[n].len = last_len;
        arr[n].cap = last_len + 1;
    }
    n++;
    *count = n;
    return arr;
}

bool klyron_string_contains(klyron_string_t *s, const char *sub) {
    if (!s->data || !sub) return false;
    return strstr(s->data, sub) != NULL;
}

bool klyron_string_starts_with(klyron_string_t *s, const char *prefix) {
    if (!s->data || !prefix) return false;
    size_t plen = strlen(prefix);
    if (plen > s->len) return false;
    return memcmp(s->data, prefix, plen) == 0;
}

bool klyron_string_ends_with(klyron_string_t *s, const char *suffix) {
    if (!s->data || !suffix) return false;
    size_t slen = strlen(suffix);
    if (slen > s->len) return false;
    return memcmp(s->data + s->len - slen, suffix, slen) == 0;
}

void klyron_string_replace(klyron_string_t *s, const char *old, const char *new_) {
    if (!s->data || !old || !new_) return;
    size_t old_len = strlen(old);
    size_t new_len = strlen(new_);
    if (old_len == 0) return;
    char *p = s->data;
    size_t count = 0;
    while ((p = strstr(p, old)) != NULL) { count++; p += old_len; }
    if (count == 0) return;
    size_t result_len = s->len + count * (new_len - old_len);
    char *result = (char *)malloc(result_len + 1);
    if (!result) return;
    char *r = result;
    char *q = s->data;
    while (count--) {
        p = strstr(q, old);
        size_t seg = (size_t)(p - q);
        memcpy(r, q, seg);
        r += seg;
        memcpy(r, new_, new_len);
        r += new_len;
        q = p + old_len;
    }
    size_t remaining = strlen(q);
    memcpy(r, q, remaining);
    r[remaining] = '\0';
    free(s->data);
    s->data = result;
    s->len = result_len;
    s->cap = result_len + 1;
}

void klyron_string_clear(klyron_string_t *s) {
    if (s->data) s->data[0] = '\0';
    s->len = 0;
}

klyron_string_t klyron_string_from_bytes(const uint8_t *bytes, size_t len) {
    klyron_string_t str;
    str.data = (char *)malloc(len + 1);
    if (str.data) {
        memcpy(str.data, bytes, len);
        str.data[len] = '\0';
        str.len = len;
        str.cap = len + 1;
    } else {
        str.len = 0;
        str.cap = 0;
    }
    return str;
}

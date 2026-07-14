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

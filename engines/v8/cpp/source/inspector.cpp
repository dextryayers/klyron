#include "klyron_v8.h"
#include "cpp/impl/internal.h"

#include <cstring>
#include <string>

static bool g_inspector_enabled = false;

int klyron_v8_inspector_new(klyron_v8_isolate_t* isolate) {
    (void)isolate;
    return -1;
}

void klyron_v8_inspector_dispose(int inspector_id) {
    (void)inspector_id;
}

int klyron_v8_inspector_connect(int inspector_id, const char* url) {
    (void)inspector_id;
    (void)url;
    return -1;
}

void klyron_v8_inspector_disconnect(int session_id) {
    (void)session_id;
}

int klyron_v8_inspector_dispatch(int session_id, const char* message,
                                 char* out_response,
                                 size_t out_response_size) {
    (void)session_id;
    (void)message;
    if (out_response && out_response_size > 0) {
        out_response[0] = '\0';
    }
    return -1;
}

bool klyron_v8_inspector_is_active(void) {
    return g_inspector_enabled;
}

#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

/*
 * JSC C API does not expose a Web Inspector protocol in the public C API.
 * These stubs provide API parity.
 */

int klyron_jsc_inspector_new(klyron_jsc_engine_t* engine) {
    (void)engine;
    return -1;
}

void klyron_jsc_inspector_dispose(int inspector_id) {
    (void)inspector_id;
}

int klyron_jsc_inspector_connect(int inspector_id, const char* url) {
    (void)inspector_id;
    (void)url;
    return -1;
}

void klyron_jsc_inspector_disconnect(int session_id) {
    (void)session_id;
}

int klyron_jsc_inspector_dispatch(int session_id, const char* message,
                                   char* out_response, size_t out_response_size) {
    (void)session_id;
    (void)message;
    (void)out_response;
    (void)out_response_size;
    return -1;
}

bool klyron_jsc_inspector_is_active(void) {
    return false;
}

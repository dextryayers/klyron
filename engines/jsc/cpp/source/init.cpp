#include "klyron_jsc.h"
#include "cpp/impl/internal.h"

klyron_jsc_engine_t* klyron_jsc_init(void) {
    klyron_jsc_engine_t* engine = new (std::nothrow) klyron_jsc_engine_t();
    if (!engine) return nullptr;

    engine->group = JSContextGroupCreate();
    if (!engine->group) {
        delete engine;
        return nullptr;
    }

    engine->ctx = JSGlobalContextCreateInGroup(engine->group, nullptr);
    if (!engine->ctx) {
        JSContextGroupRelease(engine->group);
        delete engine;
        return nullptr;
    }

    return engine;
}

void klyron_jsc_shutdown(klyron_jsc_engine_t* engine) {
    delete engine;
}

#include "klyron_v8.h"
#include "cpp/impl/internal.h"

klyron_v8_context_t* klyron_v8_context_new(klyron_v8_isolate_t* isolate) {
    if (!isolate || !isolate->isolate) return nullptr;
    return new klyron_v8_context(isolate->isolate, isolate);
}

void klyron_v8_context_dispose(klyron_v8_context_t* context) {
    delete context;
}

void klyron_v8_context_enter(klyron_v8_context_t* context) {
    if (context && context->context) {
        auto iso = context->parent->isolate;
        v8::HandleScope scope(iso);
        context->context->Get(iso)->Enter();
    }
}

void klyron_v8_context_exit(klyron_v8_context_t* context) {
    if (context && context->context) {
        auto iso = context->parent->isolate;
        v8::HandleScope scope(iso);
        context->context->Get(iso)->Exit();
    }
}

klyron_v8_context_t* klyron_v8_context_new_from_snapshot(
    klyron_v8_isolate_t* isolate, klyron_v8_snapshot_t* snapshot) {
    if (!isolate || !snapshot) return nullptr;
    return new klyron_v8_context(isolate->isolate, isolate);
}

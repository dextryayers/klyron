#ifndef KLYRON_V8_TYPES_H
#define KLYRON_V8_TYPES_H

#include "klyron_v8.h"

#include <v8.h>

#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

struct klyron_v8_isolate {
    v8::Isolate* isolate;
    bool owns;
    char error_buf[KLYRON_V8_ERROR_BUF_SIZE];

    klyron_v8_isolate(v8::Isolate* iso, bool own) : isolate(iso), owns(own) {
        error_buf[0] = '\0';
    }
    ~klyron_v8_isolate() {
        if (owns && isolate) {
            isolate->Dispose();
        }
    }
};

struct klyron_v8_context {
    v8::Global<v8::Context>* context;
    klyron_v8_isolate* parent;

    klyron_v8_context(v8::Isolate* iso, klyron_v8_isolate* p)
        : context(new v8::Global<v8::Context>()), parent(p)
    {
        auto local = v8::Context::New(iso);
        context->Reset(iso, local);
    }

    klyron_v8_context(v8::Isolate* iso, klyron_v8_isolate* p, v8::Local<v8::Context> local)
        : context(new v8::Global<v8::Context>()), parent(p)
    {
        context->Reset(iso, local);
    }

    ~klyron_v8_context() {
        if (parent && parent->isolate && context) {
            context->Reset();
        }
        delete context;
    }
};

struct klyron_v8_value {
    v8::Global<v8::Value>* value;
    klyron_v8_isolate* parent;

    klyron_v8_value(v8::Isolate* iso, v8::Local<v8::Value> val, klyron_v8_isolate* p)
        : value(new v8::Global<v8::Value>()), parent(p)
    {
        value->Reset(iso, val);
    }

    ~klyron_v8_value() {
        if (parent && parent->isolate && value) {
            value->Reset();
        }
        delete value;
    }
};

struct klyron_v8_script {
    v8::Global<v8::Script>* script;
    klyron_v8_isolate* parent;

    klyron_v8_script(v8::Isolate* iso, v8::Local<v8::Script> s, klyron_v8_isolate* p)
        : script(new v8::Global<v8::Script>()), parent(p)
    {
        script->Reset(iso, s);
    }

    ~klyron_v8_script() {
        if (parent && parent->isolate && script) {
            script->Reset();
        }
        delete script;
    }
};

struct klyron_v8_module {
    v8::Global<v8::Module>* module;
    klyron_v8_isolate* parent;

    klyron_v8_module(v8::Isolate* iso, v8::Local<v8::Module> m, klyron_v8_isolate* p)
        : module(new v8::Global<v8::Module>()), parent(p)
    {
        module->Reset(iso, m);
    }

    ~klyron_v8_module() {
        if (parent && parent->isolate && module) {
            module->Reset();
        }
        delete module;
    }
};

struct klyron_v8_promise {
    v8::Global<v8::Promise::Resolver>* resolver;
    std::string pending_reason;
    klyron_v8_isolate* parent;

    klyron_v8_promise(v8::Isolate* iso, v8::Local<v8::Promise::Resolver> r, klyron_v8_isolate* p)
        : resolver(new v8::Global<v8::Promise::Resolver>()), parent(p)
    {
        resolver->Reset(iso, r);
    }

    ~klyron_v8_promise() {
        if (parent && parent->isolate && resolver) {
            resolver->Reset();
        }
        delete resolver;
    }
};

struct klyron_v8_snapshot {
    std::vector<uint8_t> data;

    explicit klyron_v8_snapshot(const uint8_t* d, size_t len) : data(d, d + len) {}
};

#endif

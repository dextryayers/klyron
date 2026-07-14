#pragma once

#include <string>
#include <vector>
#include <map>
#include <functional>
#include <memory>
#include <stdexcept>

namespace klyron {

enum class LogLevel { Debug, Info, Warn, Error };

struct Manifest {
    std::string name;
    std::string version;
    std::string description;
    std::vector<std::string> permissions;
};

struct HookResult {
    bool success;
    std::string message;
};

class Runtime {
public:
    explicit Runtime(Manifest manifest) : manifest_(std::move(manifest)) {}

    const Manifest& manifest() const { return manifest_; }

    void log(LogLevel level, const std::string& message) {
        std::string prefix;
        switch (level) {
            case LogLevel::Debug: prefix = "[debug]"; break;
            case LogLevel::Info:  prefix = "[info]";  break;
            case LogLevel::Warn:  prefix = "[warn]";  break;
            case LogLevel::Error: prefix = "[error]"; break;
        }
        fprintf(stderr, "%s: %s\n", prefix.c_str(), message.c_str());
    }

    void info(const std::string& msg) { log(LogLevel::Info, msg); }
    void warn(const std::string& msg) { log(LogLevel::Warn, msg); }
    void error(const std::string& msg) { log(LogLevel::Error, msg); }

    std::string get_env(const std::string& key) {
        const char* val = std::getenv(key.c_str());
        return val ? std::string(val) : "";
    }

private:
    Manifest manifest_;
};

using PluginInitFn = std::function<Runtime*(const Manifest&)>;

#define KLYRON_PLUGIN(name) \
    extern "C" klyron::Runtime* klyron_plugin_create(const klyron::Manifest& manifest) { \
        return new klyron::Runtime(manifest); \
    } \
    extern "C" void klyron_plugin_destroy(klyron::Runtime* runtime) { \
        delete runtime; \
    }

} // namespace klyron

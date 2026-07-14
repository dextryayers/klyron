#ifndef KLYRON_LOGGER_HPP
#define KLYRON_LOGGER_HPP

#include "klyron.hpp"
#include <ctime>
#include <mutex>

namespace klyron {

class Logger {
public:
    static Logger &instance() {
        static Logger inst;
        return inst;
    }

    void set_min_level(LogLevel l) { min_level_ = l; }
    void set_json(bool j) { json_ = j; }
    void set_color(bool c) { color_ = c; }

    void trace(const String &msg) { log(LogLevel::Trace, msg); }
    void debug(const String &msg) { log(LogLevel::Debug, msg); }
    void info(const String &msg) { log(LogLevel::Info, msg); }
    void warn(const String &msg) { log(LogLevel::Warn, msg); }
    void error(const String &msg) { log(LogLevel::Error, msg); }
    void fatal(const String &msg) { log(LogLevel::Fatal, msg); }

private:
    LogLevel min_level_ = LogLevel::Info;
    bool json_ = false;
    bool color_ = true;
    std::mutex mtx_;

    static String level_str(LogLevel l) {
        switch (l) {
            case LogLevel::Trace: return "TRACE";
            case LogLevel::Debug: return "DEBUG";
            case LogLevel::Info:  return "INFO";
            case LogLevel::Warn:  return "WARN";
            case LogLevel::Error: return "ERROR";
            case LogLevel::Fatal: return "FATAL";
        }
        return "UNKNOWN";
    }

    static String timestamp() {
        auto now = std::time(nullptr);
        char buf[24];
        std::strftime(buf, sizeof(buf), "%Y-%m-%dT%H:%M:%S", std::gmtime(&now));
        return buf;
    }

    void log(LogLevel level, const String &msg) {
        if (level < min_level_) return;
        std::lock_guard<std::mutex> lock(mtx_);
        auto ts = timestamp();
        auto lvl = level_str(level);
        if (json_) {
            std::cout << "{\"timestamp\":\"" << ts << "\",\"level\":\""
                      << lvl << "\",\"message\":\"" << msg << "\"}" << std::endl;
        } else {
            if (level == LogLevel::Error || level == LogLevel::Fatal)
                std::cerr << ts << " [" << lvl << "] " << msg << std::endl;
            else
                std::cout << ts << " [" << lvl << "] " << msg << std::endl;
        }
    }
};

} // namespace klyron

#endif

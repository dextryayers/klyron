#ifndef KLYRON_LOGGER_HPP
#define KLYRON_LOGGER_HPP

#include "klyron.hpp"
#include <ctime>
#include <mutex>
#include <fstream>

namespace klyron {

class Logger {
public:
    static Logger &instance() {
        static Logger inst;
        return inst;
    }

    void set_min_level(LogLevel l) { min_level_ = l; }
    LogLevel min_level() const { return min_level_; }
    void set_json(bool j) { json_ = j; }
    void set_color(bool c) { color_ = c; }
    void set_file(const String &path) {
        file_stream_.open(path, std::ios::app);
    }

    void trace(const String &msg) { log(LogLevel::Trace, msg); }
    void debug(const String &msg) { log(LogLevel::Debug, msg); }
    void info(const String &msg) { log(LogLevel::Info, msg); }
    void warn(const String &msg) { log(LogLevel::Warn, msg); }
    void error(const String &msg) { log(LogLevel::Error, msg); }
    void fatal(const String &msg) { log(LogLevel::Fatal, msg); }

    void trace(const String &msg, const Map<String, String> &meta) { log(LogLevel::Trace, msg, &meta); }
    void debug(const String &msg, const Map<String, String> &meta) { log(LogLevel::Debug, msg, &meta); }
    void info(const String &msg, const Map<String, String> &meta) { log(LogLevel::Info, msg, &meta); }
    void warn(const String &msg, const Map<String, String> &meta) { log(LogLevel::Warn, msg, &meta); }
    void error(const String &msg, const Map<String, String> &meta) { log(LogLevel::Error, msg, &meta); }
    void fatal(const String &msg, const Map<String, String> &meta) { log(LogLevel::Fatal, msg, &meta); }

private:
    LogLevel min_level_ = LogLevel::Info;
    bool json_ = false;
    bool color_ = true;
    std::mutex mtx_;
    std::ofstream file_stream_;

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

    void log(LogLevel level, const String &msg, const Map<String, String> *meta = nullptr) {
        if (level < min_level_) return;
        std::lock_guard<std::mutex> lock(mtx_);
        auto ts = timestamp();
        auto lvl = level_str(level);
        String output;

        if (json_) {
            std::ostringstream os;
            os << "{\"timestamp\":\"" << ts << "\",\"level\":\""
               << lvl << "\",\"message\":\"" << msg << "\"";
            if (meta) {
                os << ",\"meta\":{";
                bool first = true;
                for (const auto &[k, v] : *meta) {
                    if (!first) os << ",";
                    os << "\"" << k << "\":\"" << v << "\"";
                    first = false;
                }
                os << "}";
            }
            os << "}";
            output = os.str();
        } else {
            output = ts + " [" + lvl + "] " + msg;
        }

        if (file_stream_.is_open()) {
            file_stream_ << output << std::endl;
        }

        if (level == LogLevel::Error || level == LogLevel::Fatal)
            std::cerr << output << std::endl;
        else
            std::cout << output << std::endl;
    }
};

} // namespace klyron

#endif

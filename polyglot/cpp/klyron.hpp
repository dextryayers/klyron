#ifndef KLYRON_HPP
#define KLYRON_HPP

#include <string>
#include <vector>
#include <map>
#include <optional>
#include <functional>
#include <chrono>
#include <sstream>
#include <fstream>
#include <iostream>
#include <iomanip>
#include <cstdlib>
#include <memory>
#include <stdexcept>

namespace klyron {

using String = std::string;
template<typename T> using Opt = std::optional<T>;
template<typename K, typename V> using Map = std::map<K, V>;
template<typename T> using Vec = std::vector<T>;

struct ProcessResult {
    String stdout_data;
    String stderr_data;
    int exit_code;
    bool success;
};

struct HttpResponse {
    int status;
    String status_text;
    Map<String, String> headers;
    String body;
    bool ok() const { return status >= 200 && status < 300; }
};

struct FileInfo {
    String path;
    uint64_t size;
    bool is_dir;
    bool is_file;
    int64_t modified;
};

struct DnsRecord {
    String name;
    String record_type;
    String value;
    uint32_t ttl;
};

enum class LogLevel { Trace, Debug, Info, Warn, Error, Fatal };

} // namespace klyron

#endif

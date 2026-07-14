#ifndef KLYRON_JSON_UTIL_HPP
#define KLYRON_JSON_UTIL_HPP

#include "klyron.hpp"
#include <nlohmann/json.hpp>

namespace klyron {

using json = nlohmann::json;

class JsonUtil {
public:
    static json parse(const String &text) {
        return json::parse(text, nullptr, false);
    }

    static String stringify(const json &j, bool pretty = false) {
        return pretty ? j.dump(2) : j.dump();
    }

    static bool is_valid(const String &text) {
        return json::accept(text);
    }

    static json merge(const json &a, const json &b) {
        json result = a;
        result.update(b);
        return result;
    }

    static json deep_merge(const json &a, const json &b) {
        json result = a;
        for (auto it = b.begin(); it != b.end(); ++it) {
            if (result.contains(it.key()) && result[it.key()].is_object() && it.value().is_object()) {
                result[it.key()] = deep_merge(result[it.key()], it.value());
            } else {
                result[it.key()] = it.value();
            }
        }
        return result;
    }

    static json read_file(const String &path) {
        std::ifstream f(path);
        if (!f) return nullptr;
        std::stringstream buf;
        buf << f.rdbuf();
        return json::parse(buf.str(), nullptr, false);
    }

    static bool write_file(const String &path, const json &j, bool pretty = false) {
        std::ofstream f(path);
        if (!f) return false;
        f << (pretty ? j.dump(2) : j.dump());
        return true;
    }

    static Opt<String> get_string(const json &j, const String &key) {
        if (j.contains(key) && j[key].is_string())
            return j[key].get<String>();
        return std::nullopt;
    }

    static Opt<int> get_int(const json &j, const String &key) {
        if (j.contains(key) && j[key].is_number_integer())
            return j[key].get<int>();
        return std::nullopt;
    }

    static Opt<double> get_float(const json &j, const String &key) {
        if (j.contains(key) && j[key].is_number_float())
            return j[key].get<double>();
        return std::nullopt;
    }

    static Opt<bool> get_bool(const json &j, const String &key) {
        if (j.contains(key) && j[key].is_boolean())
            return j[key].get<bool>();
        return std::nullopt;
    }

    static Vec<String> get_keys(const json &j) {
        Vec<String> keys;
        if (j.is_object()) {
            for (auto it = j.begin(); it != j.end(); ++it)
                keys.push_back(it.key());
        }
        return keys;
    }

    static json array_from_strings(const Vec<String> &items) {
        json arr = json::array();
        for (const auto &item : items) arr.push_back(item);
        return arr;
    }

    template<typename T>
    static json from_vector(const Vec<T> &items) {
        json arr = json::array();
        for (const auto &item : items) arr.push_back(item);
        return arr;
    }
};

} // namespace klyron

#endif

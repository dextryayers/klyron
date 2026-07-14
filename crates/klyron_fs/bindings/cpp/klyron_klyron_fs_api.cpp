#include "klyron_klyron_fs_api.hpp"
#include <string>

namespace klyron {

FileSystemApi::FileSystemApi() {}

std::string FileSystemApi::version() const {
    return "klyron_fs 0.1.0";
}

bool FileSystemApi::ping() {
    return true;
}

}

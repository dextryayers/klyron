#include "klyron_klyron_fs_errors.hpp"
#include <string>

namespace klyron {

std::string FileSystemError::error_string() const {
    return what();
}

}

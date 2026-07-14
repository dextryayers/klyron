#ifndef KLYRON_KLYRON_NODE_FFI_HPP
#define KLYRON_KLYRON_NODE_FFI_HPP

extern "C" {
    const char* klyron_node_version();
}

inline const char* klyron_node_version_str() { return klyron_node_version(); }

#endif

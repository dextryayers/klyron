#ifndef KLYRON_AI_BINDINGS_API_HPP
#define KLYRON_AI_BINDINGS_API_HPP

#include "types.hpp"
#include "config.hpp"

namespace klyron {

class Klyron::AiApi {
public:
  Klyron::AiApi();
  ~Klyron::AiApi();

  Klyron::AiResult process(const std::string& input);
  std::string version() const;
  bool ping();

private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace klyron

#endif

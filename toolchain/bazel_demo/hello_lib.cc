#include "bazel_demo/hello_lib.h"

#include <string>
#include <iostream>

namespace toolchain::bazel_demo {
int Hello::print_statement(void) {
  std::cout << "Hello World " << statement_ << std::endl;
  return 0;
}
}  // toolchain::bazel_demo

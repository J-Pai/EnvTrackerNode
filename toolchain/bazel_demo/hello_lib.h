#ifndef TOOLCHAIN_BAZEL_DEMO_HELLO_LIB_H_
#define TOOLCHAIN_BAZEL_DEMO_HELLO_LIB_H_

#include <string>

namespace toolchain::bazel_demo {
class Hello final {
 public:
    Hello(std::string statement) : statement_(statement) {}
    int print_statement(void);
 private:
    std::string statement_;
};
}  // toolchain::bazel_demo

#endif  // TOOLCHAIN_BAZEL_DEMO_HELLO_LIB_H_

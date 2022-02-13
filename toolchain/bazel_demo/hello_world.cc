#include "toolchain/bazel_demo/hello_lib.h"

namespace {
using ::toolchain::bazel_demo::Hello;
}

int main(int argc, char** argv) {
  Hello statement_generator("This is a statement");
  statement_generator.print_statement();
  return 0;
}

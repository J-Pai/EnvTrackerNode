## Cross Compilation on Ubuntu 20.04 (x86_64) to RPi Ubuntu 20.04 (aarch64)
This section describes the steps necessary to build the C++ code in an Ubuntu
20.04 (x86_64) environment.

On the main/host machine (Ubuntu 20.04 - x86_64), install the following
dependencies.

```
[sudo] apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
```

From the grpc repository:

```
mkdir -p cmake/rpi_build
cd cmake/rpi_build
cmake ../.. -DCMAKE_TOOLCHAIN_FILE=/home/jpai/Documents/EnvTrackerNode/toolchain/ubuntu_rpi_aarch64_toolchain.cmake \
-DCMAKE_BUILD_TYPE=Release \
-DCMAKE_INSTALL_PREFIX=/home/jpai/grpc_aarch64/grpc_install
make -j2
make install
```

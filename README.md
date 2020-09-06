# EnvTrackerNode - The Environment Tracking Server

Raspberry Pi project for creating a central node meant to track environmental conditions.

## Tracks
- Movement
  - Video recording
  - Live view
  - Uploading to video repository
- Temperature
  - °F/°C
  - Historical tracking
- Humidity
  - RH
  - Historical tracking

## Hardware Requirements
- Raspberry Pi 4 (Ubuntu 20.04)
  - Tested on 8gb version
  - aarch64
- USB Camera
- Sense HAT
  - Temperature and Humidity
  - LED Screen

## Installation and Setup
Do the following steps on the Raspberry Pi.
1) Setup Ubuntu 20.04 for Raspberry Pi.
2) **IMPORTANT**: Before plugging in the SD add the following lines to the
   usercfg.txt file in `boot`.

   ```
   hdmi_force_hotplug=1 # Allows RPi to boot in headless mode with Sensor HAT installed.
   dtparam=i2c_arm=on   # Enables auto loading of i2c module.
   ```

3) Install the SD card and turn on the Raspberry Pi.
3) Install build tools.

   ```
   [sudo] apt install python3 python3-dev python3-pip \
                      build-essential autoconf libtool \
                      pkg-config cmake libssl-dev \
                      i2c-tools
   ```

4) Add the following line to `/etc/modules`:

   ```
   i2c-dev
   ```

5) Create the file `/etc/udev/rules.d/99-i2c.rules` with the following contents:

   ```
   KERNEL=="i2c-[0-7]",MODE="0666"
   ```

   This will ensure that the i2c devices are accessible by all users (without
   the need for sudo).

7) Create the file `/etc/modprobe.d/blacklist-industialio.conf` with the
   following contents:

   ```
   blacklist st_magn_spi
   blacklist st_pressure_spi
   blacklist st_sensors_spi
   blacklist st_pressure_i2c
   blacklist st_magn_i2c
   blacklist st_pressure
   blacklist st_magn
   blacklist st_sensors_i2c
   blacklist st_sensors
   blacklist industrialio_triggered_buffer
   blacklist industrialio
   ```

   This ensures the Industial I/O Core module is not loaded. The modules
   blacklisted here takes over the pressure and magnetic sensors of the Sense
   HAT device which prevents other applications (like this one) from using those
   sensors.

8) Reboot the Raspberry Pi.
9) Confirm that the i2c module is loaded: `ls /dev/i2c-1`.
10) Confirm the sense-hat i2c devices can be enumerated: `i2cdetect -y 1`

   ```
        0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
   00:          -- -- -- -- -- -- -- -- -- -- -- -- --
   10: -- -- -- -- -- -- -- -- -- -- -- -- 1c -- -- --
   20: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
   30: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
   40: -- -- -- -- -- -- UU -- -- -- -- -- -- -- -- --
   50: -- -- -- -- -- -- -- -- -- -- -- -- 5c -- -- 5f
   60: -- -- -- -- -- -- -- -- -- -- 6a -- -- -- -- --
   70: -- -- -- -- -- -- -- --
   ```

### Sense HAT Setup
Do the following steps on the Raspberry Pi.
1) Clone the following repositories:
   - https://github.com/RPi-Distro/RTIMULib
   - https://github.com/astro-pi/python-sense-hat
2) Install numpy from apt.

   ```
   [sudo] apt install python3-numpy`
   ```

3) Build and install RTIMULib.
   - `cd` to RTIMULib
   - From inside the RTIMULib directory, `cd ./Linux/python`.
   - `python3 setup.py build`
   - `sudo python3 setup.py install`
4) Build and install python-sense-hat.
   - `cd` to python-sense-hat.
   - `python3 setup.py build`
   - `sudo python3 setup.py install`
5) Run the sense\_hat\_demo.py to test setup. You should see the current
   temperature and humidity scroll across the LED matrix on the installed Sense
   HAT.
6) If the demo does not work, try to reload the rpisense_fb module.

   ```
   sudo rmmod rpisense_fb
   sudo modeprobe rpisense_fb
   ```

   Problems could include the following:
   - Sensor fails to initialize
   - Sensors read 0C and/or 0%rH
   - Application requires sudo


### gRPC (v1.31.1) Setup
Based on the steps [here](https://github.com/grpc/grpc/blob/master/BUILDING.md).

Do the following on the Raspberry Pi.

1) Clone and init the gRPC repository.

   ```
   git clone -b v1.31.1 https://github.com/grpc/grpc
   cd grpc
   git submodule update --init
   ```

2) After cloning and initializing the repository, build and install gRPC.

   ```
   mkdir -p cmake/build
   cd cmake/build
   cmake ../.. -DCMAKE_BUILD_TYPE=Release \
               -DgRPC_INSTALL=ON          \
               -DgRPC_BUILD_TESTS=OFF     \
               -DgRPC_SSL_PROVIDER=package
   make -j
   sudo make install
   ```

<!---
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
--->

## Wire Schematic

## Architecture Overview

## Raspberry Pi Optimizations

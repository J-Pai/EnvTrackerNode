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

## Wire Schematic
![rpi schematic](./.md/rpi_sketch_bb.png "Schematic")

### Hardware Setup Recommendations
While the Sense HAT can be directly attached to the GPIO pins of the Raspberry
Pi, it is generally recommended that the two components are separated. This is
why the wire schematic only shows the bare minimum GPIO connections.

As the Raspberry Pi heats up, the temperature of the Sense HAT PCB also
increases which throws off the accuracy of thermometers. Separating the Sense
HAT from the body of the Raspberry Pi will greatly increase the reliability and
accuracy of sensor readings.

If you still want to directly connect the Sense HAT to the Raspberry Pi take a
look at this [article](https://github.com/initialstate/wunderground-sensehat/wiki/Part-3.-Sense-HAT-Temperature-Correction#a-much-less-accurate-but-compact-solution).

## Architecture Overview

## Installation and Setup
### Development Setup
Development is assumed to be done on Ubuntu 20.04 (x86_64).

1) Install development tools and dependencies.

   ```bash
   sudo apt install python3 python3-dev python3-pip python3-venv  \
                    build-essential autoconf libtool bear         \
                    pkg-config cmake libssl-dev libsasl2-dev      \
                    openssl libcurl4-openssl-dev libc++-dev       \
                    libc++abi-dev

   # To support cross-compilation to AArch64:
   sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu

   python3 -m pip install --user cpplint
   ```

2) Install Bazel. Commands based on [Installing Bazel on Ubuntu](https://docs.bazel.build/versions/5.0.0/install-ubuntu.html).

   ```bash
   sudo apt install apt-transport-https curl gnupg
   curl -fsSL https://bazel.build/bazel-release.pub.gpg | gpg --dearmor > bazel.gpg
   sudo mv bazel.gpg /etc/apt/trusted.gpg.d/
   echo "deb [arch=amd64] https://storage.googleapis.com/bazel-apt stable jdk1.8" | sudo tee /etc/apt/sources.list.d/bazel.list
   sudo apt update
   sudo apt install bazel
   ```

Check the individual `node` directories on how to build/deploy each `node`.

### Raspberry Pi Setup
Do the following steps on the Raspberry Pi.

1) Setup Ubuntu 20.04 for Raspberry Pi.

2) **IMPORTANT**: Before plugging in the SD add the following lines to the
usercfg.txt file in `boot`.

   ```bash
   hdmi_force_hotplug=1 # Allows RPi to boot in headless mode with Sensor HAT installed.
   dtparam=i2c_arm=on   # Enables auto loading of i2c module.
   ```

3) Install the SD card and turn on the Raspberry Pi.

4) Install runtime dependencies and tools.
   ```bash
   [sudo] apt install python3 python3-dev python3-pip python3-venv  \
                      i2c-tools openssl
   ```

5) Add the following line to `/etc/modules`:

   ```
   i2c-dev
   ```

6) Create the file `/etc/udev/rules.d/99-i2c.rules` with the following contents:

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
10) Confirm the sense-hat i2c devices can be enumerated: `i2cdetect -y 1`.

    ```bash
    $ i2cdetect -y 1
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

   ```bash
   [sudo] apt install python3-numpy
   ```

3) Build and install RTIMULib.
   - `cd` to RTIMULib
   - From inside the RTIMULib directory, `cd ./Linux/python`.
   - `python3 setup.py build`
   - `[sudo] python3 setup.py install`

4) Build and install python-sense-hat.
   - `cd` to python-sense-hat.
   - `python3 setup.py build`
   - `[sudo] python3 setup.py install`

5) Run `scripts/sense_hat_demo.py` to test setup. You should see the current
temperature and humidity scroll across the LED matrix on the installed Sense
HAT.

6) If the demo does not work, try to reload the rpisense_fb module.

   ```bash
   sudo rmmod rpisense_fb
   sudo modeprobe rpisense_fb
   ```

   Problems could include the following:
   - Sensor fails to initialize
   - Sensors read 0C and/or 0%rH
   - Application requires sudo

#### Sense HAT Emulation
If you are looking to develop/test on a different (non-raspberry pi) machine,
make sure to install the `sense_emu` package to emulate the Sense HAT.

```bash
python3 -m pip install sense_emu
```

### gRPC (v1.31.1) Setup
Based on the steps [here](https://github.com/grpc/grpc/blob/master/BUILDING.md).

Do the following on the Raspberry Pi.

1) Clone and init the gRPC repository.

   ```bash
   git clone -b v1.31.1 https://github.com/grpc/grpc
   cd grpc
   git submodule update --init
   ```

2) After cloning and initializing the repository, build and install gRPC.

   ```bash
   mkdir -p cmake/build
   cd cmake/build
   cmake ../.. -DCMAKE_BUILD_TYPE=Release       \
               -DgRPC_INSTALL=ON                \
               -DgRPC_BUILD_TESTS=OFF           \
               -DgRPC_SSL_PROVIDER=package
   make -j2
   sudo make install
   ```

## Raspberry Pi Optimizations
### Use tmpfs for temporary files
Add the following lines to `/etc/fstab` and reboot the Raspberry Pi.

```bash
tmpfs    /tmp    tmpfs    defaults,noatime,nosuid,size=100m    0 0
tmpfs    /var/tmp    tmpfs    defaults,noatime,nosuid,size=100m    0 0
tmpfs    /var/log    tmpfs    defaults,noatime,nosuid,mode=0755,size=100m    0 0
tmpfs    /var/run    tmpfs    defaults,noatime,nosuid,mode=0755,size=2m    0 0
tmpfs    /var/spool/mqueue    tmpfs    defaults,noatime,nosuid,mode=0700,gid=12,size=30m    0 0
```

After rebooting, verify that the temporary file directories are now using tmpfs.

```bash
$ df -h
Filesystem      Size  Used Avail Use% Mounted on
...
tmpfs           100M   84K  100M   1% /tmp
tmpfs           100M  340K  100M   1% /var/log
tmpfs            30M     0   30M   0% /var/spool/mqueue
tmpfs           100M     0  100M   0% /var/tmp
...
```

### Setup Static Ethernet

Personal development setup involves a direct ethernet connection between
the development computer and the Raspberry Pi. Internet connection comes from
WiFi.

To enable eth0 interface on boot, update `/etc/netplan/50-cloud-init.yaml` to
match the following:

```
network:
    version: 2
    renderer: networkd
    wifis:
        wlan0:
            access-points:
                <WIFI>:
                    password: <WIFI_PWD>
            dhcp4: true
            optional: true

    ethernets:
        eth0:
            addresses: [192.168.10.10/24]
            dhcp4: false                      # static IP assignment
```

The run:

```
sudo netplan --debug apply
```

### Disable Auto-updates

```
# /etc/apt/apt.conf.d/20auto-upgrades
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "0";
```

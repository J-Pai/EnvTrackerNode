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
- USB Camera
- Sense HAT
  - Temperature and Humidity
  - LED Screen

## Installation and Setup
1) Setup Ubuntu 20.04 for Raspberry Pi.
2) **IMPORTANT**: Before plugging in the SD add the following lines to the usercfg.txt file in `boot`.
   ```
   hdmi_force_hotplug=1 # Allows RPi to boot in headless mode with Sensor HAT installed.
   dtparam=i2c_arm=on   # Enables auto loading of i2c module.
   ```
3) Install the SD card and turn on the Raspberry Pi.
3) Install build tools.
   ```
   [sudo] apt install python3 python3-dev python3-pip build-essential autoconf libtool pkg-config cmake
   ```
4) Add the following line to `/etc/modules`:
   ```
   i2c-dev
   ```
5) Reboot the Raspberry Pi.
6) Confirm that the i2c module is loaded: `ls /dev/i2c-1`.

### Sense HAT Setup
1) Clone the following repositories:
   - https://github.com/RPi-Distro/RTIMULib
   - https://github.com/astro-pi/python-sense-hat
2) Install numpy from apt.
   `[sudo] apt install python3-numpy`
3) Build and install RTIMULib.
   - `cd` to RTIMULib
   - From inside the RTIMULib directory, `cd ./Linux/python`.
   - `python3 setup.py build`
   - `sudo python3 setup.py install`
4) Build and install python-sense-hat.
   - `cd` to python-sense-hat.
   - `python3 setup.py build`
   - `sudo python3 setup.py install`
5) Run the sense\_hat\_demo.py to test setup. You should see `Hello World!`
   scroll across the LED matrix on the installed Sense HAT.

### gRPC Setup
Following the steps [here](https://github.com/grpc/grpc/blob/master/BUILDING.md).

## Wire Schematic

## Architecture Overview

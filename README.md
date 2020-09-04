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
3) Install build tools.
   ```
   [sudo] apt install python3 python3-dev python3-pip build-essential autoconf libtool pkg-config cmake
   ```
### Sense HAT Setup
1) Clone the following repositories:
   - https://github.com/RPi-Distro/RTIMULib
   - https://github.com/astro-pi/python-sense-hat
2) Install numpy from apt. 
   `[sudo] apt install python3-numpy`
3) Build and install RTIMULib.
   a. `cd` to RTIMULib
   b. From inside the RTIMULib directory, `cd Linux/python`.
   c. `python3 setup.py build`
   d. `sudo python3 setup.py install`
4) Build and install python-sense-hat.
   a. `cd` to python-sense-hat.
   b. `python3 setup.py build`
   c. `sudo python3 setup.py install`
5) Run the sense\_hat\_demo.py to test setup. You should see `Hello World!` 
   scroll across the LED matrix on the installed Sense HAT.

### gRPC Setup
Following the steps [here](https://github.com/grpc/grpc/blob/master/BUILDING.md).

## Wire Schematic

## Architecture Overview

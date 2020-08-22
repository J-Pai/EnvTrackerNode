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
- Raspberry Pi 4
  - Tested on 8gb version
- USB Camera
- DHT11 Sensor
  - Temperature and Humidity

## Installation and Setup
1) Install gRPC prerequisites [here](https://github.com/grpc/grpc/blob/master/BUILDING.md#pre-requisites)
   ```
   [sudo] apt-get install build-essential autoconf libtool pkg-config cmake
   ```

## Wire Schematic

## Architecture Overview

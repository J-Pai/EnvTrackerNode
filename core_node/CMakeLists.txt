cmake_minimum_required(VERSION 3.5.1)

project(EnvTrackerCoreNode C CXX)

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++17")

# Prior to running cmake, gRPC should already have been installed on the system.
# This section will attempt to locate the gRPC dependencies via find_package().

# Find protobuf installation.
# Looks for protobuf-config.cmake file installed by Protobuf's cmake installation.

set(protobuf_MODULE_COMPATIBLE TRUE)
find_package(Protobuf CONFIG REQUIRED)
message(STATUS "Using protobuf ${protobuf_VERSION}")

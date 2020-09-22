cmake_minimum_required(VERSION 3.16.3)

project(EnvTrackerCoreNode C CXX)

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++17")

# Prior to running cmake, gRPC should already have been installed on the system.
# This section will attempt to locate the gRPC dependencies via find_package().

# Find protobuf installation.
# Looks for protobuf-config.cmake file installed by Protobuf's cmake installation.
set(protobuf_MODULE_COMPATIBLE TRUE)
find_package(Protobuf CONFIG REQUIRED)
message(STATUS "Using protobuf ${protobuf_VERSION}")

set(_PROTOBUF_LIBPROTOBUF protobuf::libprotobuf)
set(_REFLECTION gRPC::grpc++_reflection)
if(CMAKE_CROSSCOMPILING)
  find_program(_PROTOBUF_PROTOC protoc)
else()
  set(_PROTOBUF_PROTOC $<TARGET_FILE:protobuf::protoc>)
endif()

# Find gRPC installation
# Looks for gRPCConfig.cmake file installed by gRPC's cmake installation.
find_package(gRPC CONFIG REQUIRED)
message(STATUS "Using gRPC ${gRPC_VERSION}")

set(_GRPC_GRPCPP gRPC::grpc++)
if(CMAKE_CROSSCOMPILING)
  find_program(_GRPC_CPP_PLUGIN_EXECUTABLE grpc_cpp_plugin)
else()
  set(_GRPC_CPP_PLUGIN_EXECUTABLE $<TARGET_FILE:gRPC::grpc_cpp_plugin>)
endif()

# Proto file
get_filename_component(core_node_proto "../../protos/core_node.proto" ABSOLUTE)
get_filename_component(core_node_proto_path "${core_node_proto}" DIRECTORY)
# Generated proto sources
set(core_node_proto_srcs "${CMAKE_CURRENT_BINARY_DIR}/core_node.pb.cc")
set(core_node_proto_hdrs "${CMAKE_CURRENT_BINARY_DIR}/core_node.pb.h")
set(core_node_grpc_srcs "${CMAKE_CURRENT_BINARY_DIR}/core_node.grpc.pb.cc")
set(core_node_grpc_hdrs "${CMAKE_CURRENT_BINARY_DIR}/core_node.grpc.pb.h")
add_custom_command(
  OUTPUT "${core_node_proto_srcs}" "${core_node_proto_hdrs}"
    "${core_node_grpc_srcs}" "${core_node_grpc_hdrs}"
  COMMAND ${_PROTOBUF_PROTOC}
  ARGS --grpc_out "${CMAKE_CURRENT_BINARY_DIR}"
    --cpp_out "${CMAKE_CURRENT_BINARY_DIR}"
    -I "${core_node_proto_path}"
    --plugin=protoc-gen-grpc="${_GRPC_CPP_PLUGIN_EXECUTABLE}"
    "${core_node_proto}"
    DEPENDS "${core_node_proto}")

# Include generated *.pb.h files
include_directories("${CMAKE_CURRENT_BINARY_DIR}")

foreach (_library ssl_key_cert oauth2_token_processor)
  add_library(${_library} "${_library}.cc" "${_library}.h")
  target_link_libraries(${_library}
    ${_REFLECTION}
    ${_GRPC_GRPCPP}
    ${_PROTOBUF_LIBPROTOBUF})
  list(APPEND _local_libraries ${_library})
endforeach()

foreach(_target core_server basic_client)
  add_executable(${_target} "${_target}.cc"
    ${core_node_proto_srcs}
    ${core_node_grpc_srcs})
  target_link_libraries(${_target}
    ${_local_libraries}
    ${_REFLECTION}
    ${_GRPC_GRPCPP}
    ${_PROTOBUF_LIBPROTOBUF})
endforeach()

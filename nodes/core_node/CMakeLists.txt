cmake_minimum_required(VERSION 3.16.3)

project(EnvTrackerCoreNode
  LANGUAGES C CXX
  VERSION 0.1)

include(FetchContent)

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

# Find oauth2_cli application
# To be used with basic_client for testing Google OAuth2 token validation logic.
find_program(_OAUTH2_CLI_EXECUTABLE oauth2_cli)
if(_OAUTH2_CLI_EXECUTABLE)
  message(STATUS "Using oauth2_cli ${_OAUTH2_CLI_EXECUTABLE}")
else()
  unset(_OAUTH2_CLI_EXECUTABLE NULL)
  message(WARNING "oauth2_cli not available. Please install from scripts/oauth2_cli.")
endif()

# Install JSON library for parsing OAuth2 token.
FetchContent_Declare(json
  GIT_REPOSITORY https://github.com/nlohmann/json.git
  GIT_TAG v3.9.1)
FetchContent_MakeAvailable(json)

FetchContent_Declare(oauth2_cli
  GIT_REPOSITORY https://github.com/J-Pai/oauth2_cli.git)
FetchContent_MakeAvailable(oauth2_cli)

# Include generated *.pb.h files
include_directories("${CMAKE_CURRENT_BINARY_DIR}")

# Build utility libraries.
foreach (_library ssl_key_cert oauth2_token_processor)
  add_library(${_library} "src/${_library}.cc" "src/${_library}.h")
  target_link_libraries(${_library}
    ${_REFLECTION}
    ${_GRPC_GRPCPP}
    ${_PROTOBUF_LIBPROTOBUF})
  list(APPEND _local_libraries ${_library})
endforeach()

# Build core_server and basic_client executables.
foreach(_target core_server basic_client)
  add_executable(${_target} "src/${_target}.cc"
    ${core_node_proto_srcs}
    ${core_node_grpc_srcs})
  target_link_libraries(${_target}
    ${_local_libraries}
    ${_REFLECTION}
    ${_GRPC_GRPCPP}
    ${_PROTOBUF_LIBPROTOBUF}
    nlohmann_json::nlohmann_json)
endforeach()

target_compile_definitions(basic_client
  PRIVATE
  -DOAUTH2_CLI=${_OAUTH2_CLI_EXECUTABLE})

add_custom_target(run
  COMMAND core_server
  DEPENDS core_server
  WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})

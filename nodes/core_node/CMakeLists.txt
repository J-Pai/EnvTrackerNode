cmake_minimum_required(VERSION 3.16.3)

project(EnvTrackerCoreNode
  LANGUAGES C CXX
  VERSION 0.1)

include(FetchContent)
include(ExternalProject)

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

# Install JSON library for parsing OAuth2 token.
FetchContent_Declare(json
  GIT_REPOSITORY https://github.com/nlohmann/json.git
  GIT_TAG v3.9.1)
set(JSON_BuildTests OFF CACHE INTERNAL "")
FetchContent_MakeAvailable(json)
message(STATUS "Using nlohmann::json v3.9.1")

# Find oauth2_cli application
# To be used with basic_client for testing Google OAuth2 token validation logic.
FetchContent_Declare(oauth2_cli
  GIT_REPOSITORY https://github.com/J-Pai/oauth2_cli.git
  GIT_TAG v0.1.4)
FetchContent_MakeAvailable(oauth2_cli)
message(STATUS "Using oauth2_cli v0.1.4")

# Find curlpp library for HTTPS client on core_server.
find_package(CURL REQUIRED)
message(STATUS "Using curl ${CURL_VERSION_STRING}")
FetchContent_Declare(curlpp
  GIT_REPOSITORY https://github.com/jpbarrette/curlpp.git
  GIT_TAG v0.8.1)
FetchContent_MakeAvailable(curlpp)
message(STATUS "Using curlpp v0.8.1")

# Find mongoc, bson, and mongocxx library for use as MongoDB connector/driver.
ExternalProject_Add(libmongoc
  GIT_REPOSITORY https://github.com/mongodb/mongo-c-driver.git
  GIT_TAG 1.17.0
  GIT_PROGRESS 1
  GIT_SHALLOW 1
  UPDATE_COMMAND "" # Prevents a rebuild on every make.
  SOURCE_DIR "${CMAKE_BINARY_DIR}/libmongoc"
  BINARY_DIR "${CMAKE_BINARY_DIR}/libmongoc-build"
  INSTALL_DIR "${CMAKE_BINARY_DIR}/libmongoc-install"
  CMAKE_CACHE_ARGS
  -DCMAKE_BUILD_TYPE:STRING=Release
  -DCMAKE_INSTALL_PREFIX:PATH=${CMAKE_BINARY_DIR}/libmongoc-install
  -DENABLE_TESTS:BOOL=OFF
  -DENABLE_STATIC:BOOL=OFF
  -DENABLE_EXAMPLES:BOOL=OFF
  -DENABLE_EXTRA_ALIGNMENT:BOOL=OFF)

ExternalProject_Add(libmongocxx
  GIT_REPOSITORY  "https://github.com/mongodb/mongo-cxx-driver.git"
  GIT_TAG r3.6.0
  GIT_PROGRESS 1
  GIT_SHALLOW 1
  UPDATE_COMMAND "" # Prevents a rebuild on every make.
  SOURCE_DIR "${CMAKE_BINARY_DIR}/libmongocxx"
  BINARY_DIR "${CMAKE_BINARY_DIR}/libmongocxx-build"
  INSTALL_DIR "${CMAKE_BINARY_DIR}/libmongocxx-install"
  CMAKE_CACHE_ARGS
  -DCMAKE_BUILD_TYPE:STRING=Release
  -DCMAKE_PREFIX_PATH:PATH=${CMAKE_BINARY_DIR}/libmongoc-install
  -DCMAKE_INSTALL_PREFIX:PATH=${CMAKE_BINARY_DIR}/libmongocxx-install
  -DENABLE_TESTS:BOOL=OFF
  -DENABLE_EXAMPLES:BOOL=OFF
  -DBSONCXX_POLY_USE_BOOST:BOOL=OFF
  -DBSONCXX_POLY_USE_MNMLSTC:BOOL=ON
  -DBSONCXX_POLY_USE_STD:BOOL=OFF
  DEPENDS libmongoc)
set(_LIBMONGOCXX_LIBRARIES
  "${CMAKE_BINARY_DIR}/libmongocxx-build/src/bsoncxx/libbsoncxx.so"
  "${CMAKE_BINARY_DIR}/libmongocxx-build/src/mongocxx/libmongocxx.so")
set(_LIBMONGOCXX_INCLUDE_DIRS
  "${CMAKE_BINARY_DIR}/libmongocxx-install/include/bsoncxx/v_noabi/"
  "${CMAKE_BINARY_DIR}/libmongocxx-install/include/mongocxx/v_noabi/")
message(STATUS "Using mongocxx r3.6.0")

# Include generated *.pb.h files
include_directories("${CMAKE_CURRENT_BINARY_DIR}")

# Build utility libraries.
foreach (_library credentials_utility oauth2_token_processor)
  add_library(${_library} "src/${_library}.cc" "src/${_library}.h")
  add_dependencies(${_library} libmongocxx)
  target_include_directories(${_library}
    PUBLIC
    "${curlpp_SOURCE_DIR}/include"
    "${_LIBMONGOCXX_INCLUDE_DIRS}")
  target_link_libraries(${_library}
    ${_REFLECTION}
    ${_GRPC_GRPCPP}
    ${_PROTOBUF_LIBPROTOBUF}
    ${_LIBMONGOCXX_LIBRARIES}
    nlohmann_json::nlohmann_json
    curlpp)
  list(APPEND _local_libraries ${_library})
endforeach()

target_compile_definitions(credentials_utility
  PRIVATE
  -DOAUTH2_CLI=${oauth2_cli_BINARY_DIR}/oauth2_cli)

# Build core_server and basic_client executables.
foreach(_target core_server basic_client)
  add_executable(${_target} "src/${_target}.cc"
    ${core_node_proto_srcs}
    ${core_node_grpc_srcs})
  target_link_libraries(${_target}
    ${_REFLECTION}
    ${_GRPC_GRPCPP}
    oauth2_token_processor
    credentials_utility)
endforeach()

#include <iostream>
#include <memory>
#include <string>

#include <grpcpp/grpcpp.h>
#include <grpcpp/health_check_service_interface.h>
#include <grpcpp/ext/proto_server_reflection_plugin.h>

#include "core_node.grpc.pb.h"

class GreeterServiceImpl final : public corenode::Greeter::Service {
  grpc::Status SayHello(grpc::ServerContext* context, const corenode::HelloRequest* request,
      corenode::HelloReply* reply) override {
    std::string prefix("Hello ");
    reply->set_message(prefix + request->name());
    return grpc::Status::OK;
  }
};

void RunServer() {
  std::string server_address("0.0.0.0:50051");
  GreeterServiceImpl greeterService;

  grpc::EnableDefaultHealthCheckService(true);
  grpc::reflection::InitProtoReflectionServerBuilderPlugin();
  grpc::ServerBuilder builder;
  // Listen on the given address.
  builder.AddListeningPort(server_address, grpc::InsecureServerCredentials());
  // Register "greeterService" as the instance through which communications with
  // clients will be done.
  builder.RegisterService(&greeterService);
  // Assemble the server.
  std::unique_ptr<grpc::Server> server(builder.BuildAndStart());
  std::cout << "Server listening on " << server_address << std::endl;

  // Wait for server to shutdown.
  server->Wait();
}

int main(int argc, char** argv) {
  RunServer();
  return 0;
}

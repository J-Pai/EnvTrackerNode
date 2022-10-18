#include <iostream>
#include <memory>
#include <string>

#include <grpcpp/grpcpp.h>

#include "proto/core_node.grpc.pb.h"

namespace envtrackernode {
class CoreNodeServiceImpl final : public CoreNode::Service {
  grpc::Status SayHello(grpc::ServerContext* context,
                        const HelloRequest* request,
                        HelloReply* reply) override {
    return grpc::Status::OK;
  }
};

void RunServer(void) {
  std::string server_address("0.0.0.0:50051");
  CoreNodeServiceImpl service;

  grpc::ServerBuilder builder;
  builder.AddListeningPort(server_address, grpc::InsecureServerCredentials());
  builder.RegisterService(&service);
  std::unique_ptr<grpc::Server> server(builder.BuildAndStart());
  std::cout << "Server listening on " << server_address << std::endl;
  server->Wait();
}
}  // envtrackernode

int main(int argc, char** argv) {
  envtrackernode::RunServer();

  return 0;
}

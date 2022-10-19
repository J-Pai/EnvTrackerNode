#include <grpcpp/client_context.h>
#include <iostream>
#include <memory>
#include <string>

#include <glog/logging.h>
#include <grpcpp/grpcpp.h>

#include "proto/core_node.grpc.pb.h"

namespace envtrackernode::democlient {
  class Client {
   public:
     Client(std::shared_ptr<grpc::Channel> channel)
       : stub_(CoreNode::NewStub(channel)) {}
     std::string SayHello(const std::string user) {
      HelloRequest request;
      request.set_name(user);

      HelloReply response;
      grpc::ClientContext ctx;

      grpc::Status status = stub_->SayHello(&ctx, request, &response);
      if (status.ok()) {
        return response.message();
      } else {
        LOG(ERROR) << status.error_code() << ": " << status.error_message();
        return "RPC failed";
      }
    }
   private:
    std::unique_ptr<CoreNode::Stub> stub_;
  };
}

int main(int argc, char** argv) {
  google::InitGoogleLogging(argv[0]);
  envtrackernode::democlient::Client client(
      grpc::CreateChannel("0.0.0.0:50051",
      grpc::InsecureChannelCredentials()));
  std::string user("world");
  std::string response = client.SayHello(user);
  LOG(INFO) << "Client received: " << response << std::endl;
}

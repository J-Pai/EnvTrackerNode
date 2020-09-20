#include <iostream>
#include <memory>
#include <string>

#include <grpcpp/grpcpp.h>

#include "core_node.grpc.pb.h"
#include "ssl_key_cert.h"

class GreeterClient {
  public:
    GreeterClient(std::shared_ptr<grpc::Channel> channel)
      : stub_(corenode::Greeter::NewStub(channel)) {}
    std::string SayHello(const std::string& user) {
      corenode::HelloRequest request;
      request.set_name(user);

      corenode::HelloReply reply;

      grpc::ClientContext context;

      grpc::Status status = stub_->SayHello(&context, request, &reply);

      if (status.ok()) {
        return reply.message();
      } else {
        std::cout << status.error_code() << ": " << status.error_message()
          << std::endl;
        return "RPC failed";
      }
    }
  private:
    std::unique_ptr<corenode::Greeter::Stub> stub_;
};

int main(int argc, char** argv) {
  std::string target_str;
  std::string arg_str("--target");
  if (argc > 1) {
    std::string arg_val = argv[1];
    size_t start_pos = arg_val.find(arg_str);
    if (start_pos != std::string::npos) {
      start_pos += arg_str.size();
      if (arg_val[start_pos] == '=') {
        target_str = arg_val.substr(start_pos + 1);
      } else {
        std::cout << "The only correct argument syntax is --target=" << std::endl;
        return 1;
      }
    } else {
      std::cout << "The only correct argument syntax is --target=" << std::endl;
      return 2;
    }
  } else {
    target_str = "localhost:50051";
  }

  std::unique_ptr<corenode::SslKeyCert> sslKeyCert;
  std::shared_ptr<grpc::ChannelCredentials> credentials;

  try {
    sslKeyCert = std::unique_ptr<corenode::SslKeyCert>(new corenode::SslKeyCert);
    credentials = sslKeyCert->GenerateChannelCredentials();
  } catch (const std::runtime_error& error) {
    std::cout << "Error in SslKeyCert creation: " << error.what() << std::endl;
  }

  if (!credentials) {
    std::cout << "Credentials failed to be generated. Channel will be started using Insecure Credentials." << std::endl;
  }

  GreeterClient greeterClient(grpc::CreateChannel(
        target_str, credentials ? credentials : grpc::InsecureChannelCredentials()));
  std::string user("world");
  std::string reply = greeterClient.SayHello(user);
  std::cout << "Greeter client received: " << reply << std::endl;

  return 0;
}

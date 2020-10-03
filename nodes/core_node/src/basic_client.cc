#include <iostream>
#include <iterator>
#include <memory>

#include <grpcpp/grpcpp.h>
#include <nlohmann/json.hpp>

#include "core_node.grpc.pb.h"
#include "ssl_key_cert.h"

class CoreNodeClient {
  public:
    CoreNodeClient(std::shared_ptr<grpc::Channel> channel)
      : stub_(envtrackernode::CoreNode::NewStub(channel)) {}
    std::string SayHello(const std::string& user) {
      envtrackernode::HelloRequest request;
      request.set_name(user);

      envtrackernode::HelloReply reply;

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
    std::unique_ptr<envtrackernode::CoreNode::Stub> stub_;
};

/**
 * Extracts the target gRPC server from the commandline argument flag --target.
 * Otherwise, defaults to localhost:50051.
 *
 * @return string gRPC target, defaults to localhost:50051 if --target is not
 * specified.
 */
std::string GetTarget(int argc, char** argv) {
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
        exit(1);
      }
    } else {
      std::cout << "The only correct argument syntax is --target=" << std::endl;
      exit(2);
    }
    return target_str;
  }
  return "localhost:50051";
}

int main(int argc, char** argv) {
  std::shared_ptr<grpc::ChannelCredentials> credentials;
  std::string target_str(GetTarget(argc, argv));

  const char* environment_oauth2_token = std::getenv("OAUTH2_TOKEN");
  try {
    std::unique_ptr<corenode::SslKeyCert> sslKeyCert =
      std::unique_ptr<corenode::SslKeyCert>(new corenode::SslKeyCert);
    nlohmann::json oauth2_credential;

    if (environment_oauth2_token) {
      sslKeyCert->SetOAuthToken(std::string(environment_oauth2_token));
    }

    try {
      oauth2_credential = sslKeyCert->GetOAuthToken();
    } catch (const std::runtime_error& error) {
      std::cout << "Error in OAuth2 request: " << error.what() << std::endl;
    }

    std::shared_ptr<grpc::ChannelCredentials> tlsCredentials =
      sslKeyCert->GenerateChannelCredentials();
    if (!oauth2_credential.empty()) {
      credentials = grpc::CompositeChannelCredentials(tlsCredentials,
          std::shared_ptr<grpc::CallCredentials>(
            grpc::AccessTokenCredentials(oauth2_credential["token"])));
    } else {
      credentials = tlsCredentials;
    }
  } catch (const std::runtime_error& error) {
    std::cout << "Error in SslKeyCert creation: " << error.what() << std::endl;
  }

  if (!credentials) {
    std::cout << "Credentials failed to be generated."
      << "Channel will be started using Insecure Credentials."
      << std::endl;
  }

  CoreNodeClient coreNodeClient(grpc::CreateChannel(
        target_str, credentials ? credentials : grpc::InsecureChannelCredentials()));
  std::string user("world");
  std::string reply = coreNodeClient.SayHello(user);
  std::cout << "CoreNode client received: " << reply << std::endl;

  return 0;
}

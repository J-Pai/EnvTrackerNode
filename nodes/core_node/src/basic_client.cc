#include <iostream>
#include <memory>

#include <grpcpp/grpcpp.h>

#include "core_node.grpc.pb.h"
#include "ssl_key_cert.h"

#define STR(x) #x
#define XSTR(x) STR(x)

/**
 * Contains the path to the oauth2_cli tool that can be used to obtain a
 * Google OAuth2 access token.
 */
const std::string OAUTH2_CLI_EXE = XSTR(OAUTH2_CLI);

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

std::string get_target(int argc, char** argv) {
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

std::string request_oauth_credential() {
  const char * client_json_path = std::getenv("CLIENT_SECRET_JSON");
  if (!client_json_path) {
    throw std::runtime_error("$CLIENT_SECRET_JSON not defined.");
  }
  char resolved_path[PATH_MAX];
  realpath(client_json_path, resolved_path);
  std::string real_path(resolved_path);

  std::cout << "oauth2_cli tool specified: " << OAUTH2_CLI_EXE << std::endl;
  std::string combined_command = OAUTH2_CLI_EXE + " " + resolved_path;
  std::cout << std::endl;

  FILE* fd = popen(combined_command.c_str(), "r");
  std::array<char, 128> buffer;
  std::string result;

  while(!feof(fd)) {
    if (fgets(buffer.data(), 128, fd) != NULL) {
      result.append(buffer.data());
      std::cout << buffer.data();
    }
  }
  std::cout << std::endl;
  pclose(fd);

  return result;
}

int main(int argc, char** argv) {
  std::unique_ptr<corenode::SslKeyCert> sslKeyCert;
  std::shared_ptr<grpc::ChannelCredentials> credentials;
  std::string target_str(get_target(argc, argv));
  std::string oauth2_credentials("");

  if (OAUTH2_CLI_EXE.compare("NULL") != 0) {
    oauth2_credentials.assign(request_oauth_credential());
  }

  try {
    sslKeyCert = std::unique_ptr<corenode::SslKeyCert>(new corenode::SslKeyCert);
    std::shared_ptr<grpc::ChannelCredentials> tlsCredentials =
      sslKeyCert->GenerateChannelCredentials();
    credentials = grpc::CompositeChannelCredentials(tlsCredentials,
        std::shared_ptr<grpc::CallCredentials>(grpc::AccessTokenCredentials("abcdefg")));
  } catch (const std::runtime_error& error) {
    std::cout << "Error in SslKeyCert creation: " << error.what() << std::endl;
  }

  if (!credentials) {
    std::cout << "Credentials failed to be generated."
      << "Channel will be started using Insecure Credentials."
      << std::endl;
  }

  GreeterClient greeterClient(grpc::CreateChannel(
        target_str, credentials ? credentials : grpc::InsecureChannelCredentials()));
  std::string user("world");
  std::string reply = greeterClient.SayHello(user);
  std::cout << "Greeter client received: " << reply << std::endl;

  return 0;
}

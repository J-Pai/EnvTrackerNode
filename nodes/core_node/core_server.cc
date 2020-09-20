#include <fstream>
#include <iostream>
#include <linux/limits.h>
#include <memory>
#include <string>
#include <sstream>

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

void ReadFile(const std::string& filename, std::string& data) {
  std::ifstream file(filename.c_str(), std::ios::in);
  if (file.is_open()) {
    std::stringstream string_stream;
    string_stream << file.rdbuf();
    file.close();
    data = string_stream.str();
  }
}

std::shared_ptr<grpc::ServerCredentials> GenerateServerCredentials() {
  const char* key_path = std::getenv("SSL_KEY");
  const char* cert_path = std::getenv("SSL_CERT");
  const char* root_path = std::getenv("SSL_ROOT_CERT");
  char resolved_path[PATH_MAX];

  std::string key;
  std::string cert;
  std::string root;

  if (!key_path) {
    std::cout << "$SSL_KEY not defined." << std::endl;
    return nullptr;
  }
  realpath(key_path, resolved_path);
  ReadFile(resolved_path, key);

  if (!cert_path) {
    std::cout << "$SSL_CERT not defined." << std::endl;
    return nullptr;
  }
  realpath(cert_path, resolved_path);
  ReadFile(resolved_path, cert);

  if (!root_path) {
    std::cout << "$SSL_ROOT_CERT not defined." << std::endl;
    return nullptr;
  }
  realpath(root_path, resolved_path);
  ReadFile(resolved_path, root);

  grpc::SslServerCredentialsOptions::PemKeyCertPair key_cert = {
    key,
    cert
  };
  grpc::SslServerCredentialsOptions ssl_ops;
  ssl_ops.pem_root_certs = root;
  ssl_ops.pem_key_cert_pairs.push_back(key_cert);

  return grpc::SslServerCredentials(ssl_ops);
}

void RunServer() {
  std::string server_address("0.0.0.0:50051");

  std::shared_ptr<grpc::ServerCredentials> credentials =
    GenerateServerCredentials();

  if (!credentials) {
    std::cout << "Credentials failed to be generated. Server will be started using Insecure Credentials." << std::endl;
  }

  GreeterServiceImpl greeterService;

  grpc::EnableDefaultHealthCheckService(true);
  grpc::reflection::InitProtoReflectionServerBuilderPlugin();
  grpc::ServerBuilder builder;
  // Listen on the given address.
  builder.AddListeningPort(server_address, credentials ? credentials : grpc::InsecureServerCredentials());
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

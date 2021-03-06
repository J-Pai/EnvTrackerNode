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

#include "oauth2_token_processor.h"
#include "credentials_utility.h"

class CoreNodeServiceImpl final : public envtrackernode::CoreNode::Service {
  grpc::Status SayHello(grpc::ServerContext* context,
      const envtrackernode::HelloRequest* request,
      envtrackernode::HelloReply* reply) override {
    time_t my_time = time(NULL);
    std::cout << ctime(&my_time) << "-> Request made... " << context->peer() << std::endl;
    std::string prefix("Hello ");
    reply->set_message(prefix + request->name());
    return grpc::Status::OK;
  }
};

void RunServer(const std::string& env_json_path) {
  std::string server_address("0.0.0.0:50051");
  std::shared_ptr<grpc::ServerCredentials> credentials;

  CoreNodeServiceImpl coreNodeService;

  try {
    std::shared_ptr<corenode::CredentialsUtility> utility =
      std::shared_ptr<corenode::CredentialsUtility>(env_json_path.empty() ?
          new corenode::CredentialsUtility :
          new corenode::CredentialsUtility(env_json_path));
    std::shared_ptr<corenode::OAuth2TokenProcessor> oauth2_processor =
      std::shared_ptr<corenode::OAuth2TokenProcessor>(
          new corenode::OAuth2TokenProcessor(utility));
    credentials = utility->GenerateServerCredentials();
    credentials->SetAuthMetadataProcessor(oauth2_processor);
  } catch (const std::runtime_error& error) {
    std::cout << "Error in CredentialsUtility creation: " << error.what() << std::endl;
  }

  if (!credentials) {
    std::cout << "Credentials failed to be generated."
      " Server will be started using Insecure Credentials." << std::endl;
  }

  grpc::EnableDefaultHealthCheckService(true);
  grpc::reflection::InitProtoReflectionServerBuilderPlugin();
  grpc::ServerBuilder builder;
  // Listen on the given address.
  builder.AddListeningPort(server_address,
      credentials ? credentials : grpc::InsecureServerCredentials());
  // Register "coreNodeService" as the instance through which communications with
  // clients will be done.
  builder.RegisterService(&coreNodeService);
  // Assemble the server.
  std::unique_ptr<grpc::Server> server(builder.BuildAndStart());
  std::cout << "Server listening on " << server_address << std::endl;

  // Wait for server to shutdown.
  server->Wait();
}

int main(int argc, char** argv) {
  RunServer(corenode::CredentialsUtility::GetFlagValue("env_json", "", argc, argv));
  return 0;
}

#include <iostream>
#include <iterator>
#include <memory>

#include <grpcpp/grpcpp.h>
#include <mongocxx/client.hpp>
#include <nlohmann/json.hpp>

#include "core_node.grpc.pb.h"
#include "credentials_utility.h"

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

int main(int argc, char** argv) {
  std::shared_ptr<grpc::ChannelCredentials> credentials;
  std::string target_str(corenode::CredentialsUtility::GetFlagValue(
        "target", "localhost:50051", argc, argv));
  std::string env_json_path(corenode::CredentialsUtility::GetFlagValue(
        "env_json", "", argc, argv));
  std::string cli_oauth2_token(corenode::CredentialsUtility::GetFlagValue(
        "oauth2_token", "", argc, argv));

  std::unique_ptr<corenode::CredentialsUtility> utility =
    std::unique_ptr<corenode::CredentialsUtility>(env_json_path.empty() ?
        new corenode::CredentialsUtility :
        new corenode::CredentialsUtility(env_json_path));

  try {
    mongocxx::pool::entry client_entry = utility->GetMongoClient();
    mongocxx::cursor cursor = client_entry->list_databases();
    for (const bsoncxx::document::view& doc : cursor) {
      bsoncxx::document::element ele = doc["name"];
      std::cout << ele.get_utf8().value.to_string() << std::endl;
    }
  } catch (const std::runtime_error& error) {
    std::cout << "Error in MongoDB connection: " << error.what() << std::endl;
  }

  nlohmann::json oauth2_credential;

  if (!cli_oauth2_token.empty()) {
    utility->SetOAuthToken(std::string(cli_oauth2_token));
  }

  try {
    oauth2_credential = utility->GetOAuthToken();
  } catch (const std::runtime_error& error) {
    std::cout << "Error in OAuth2 request: " << error.what() << std::endl;
  }

  try {
    std::shared_ptr<grpc::ChannelCredentials> tlsCredentials =
      utility->GenerateChannelCredentials();
    if (!oauth2_credential.empty()) {
      credentials = grpc::CompositeChannelCredentials(tlsCredentials,
          std::shared_ptr<grpc::CallCredentials>(
            grpc::AccessTokenCredentials(oauth2_credential["token"])));
    } else {
      credentials = tlsCredentials;
    }
  } catch (const std::runtime_error& error) {
    std::cout << "Error in CredentialsUtility creation: " << error.what() << std::endl;
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

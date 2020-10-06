#include <iostream>
#include <iterator>
#include <memory>

#include <bsoncxx/builder/stream/document.hpp>
#include <grpcpp/grpcpp.h>
#include <mongocxx/client.hpp>
#include <nlohmann/json.hpp>

#include "core_node.grpc.pb.h"
#include "credentials_utility.h"
#include "oauth2_token_processor.h"

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
    mongocxx::pool::entry client_entry = utility->GetMongoClient();

    std::string token(oauth2_credential["token"]);
    nlohmann::json token_info = corenode::OAuth2TokenProcessor::GetTokenInfo(token);

    std::cout << token_info << std::endl;

    mongocxx::database database = client_entry->database(utility->GetDatabaseName());

    bsoncxx::document::value filter =
      bsoncxx::builder::basic::make_document(
          bsoncxx::builder::basic::kvp("_id", token_info["sub"].get<std::string>()));
    bsoncxx::document::value document =
      bsoncxx::builder::basic::make_document(
          bsoncxx::builder::basic::kvp("$set",
            bsoncxx::builder::basic::make_document(
              bsoncxx::builder::basic::kvp("_id", token_info["sub"].get<std::string>()),
              bsoncxx::builder::basic::kvp("email", token_info["email"].get<std::string>()))),
          bsoncxx::builder::basic::kvp("$addToSet",
              bsoncxx::builder::basic::make_document(
                bsoncxx::builder::basic::kvp("tokens",
                  bsoncxx::builder::basic::make_document(
                    bsoncxx::builder::basic::kvp("token", oauth2_credential["token"].get<std::string>()),
                    bsoncxx::builder::basic::kvp("expiration", token_info["exp"].get<std::string>()))))));

    bsoncxx::stdx::optional<mongocxx::result::update> result =
      database["envtrackernode_users"].update_one(
          filter.view(), document.view(), mongocxx::options::update{}.upsert(true));
  } catch (const std::runtime_error& error) {
    std::cout << "Error in MongoDB connection: " << error.what() << std::endl;
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

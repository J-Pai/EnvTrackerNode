#include <iostream>
#include <iterator>
#include <memory>
#include <regex>

#include <grpcpp/grpcpp.h>
#include <nlohmann/json.hpp>

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

/**
 * Extracts the target gRPC server from the commandline argument flag --target.
 * Otherwise, defaults to localhost:50051.
 *
 * @return string gRPC target, defaults to localhost:50051 if --target is not
 * specified.
 */
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

/**
 * Replaces every occurance of the string variable "from" with the string
 * variable "to".
 *
 * @param str string to do that needs replacement.
 * @param from target string to search for.
 * @param to string to replace target string with.
 */
void replace_all(std::string& str, const std::string& from, const std::string& to) {
  if (from.empty()) {
    return;
  }
  size_t pos = 0;
  while((pos = str.find(from, pos)) != std::string::npos) {
    str.replace(pos, from.length(), to);
    pos += to.length();
  }
}

/**
 * Requests an Google OAuth2 token using the oauth2_cli application.
 *
 * @return {@link nlohmann::json}
 */
nlohmann::json request_oauth_credential() {
  const char * client_json_path = std::getenv("CLIENT_SECRET_JSON");
  if (!client_json_path) {
    throw std::runtime_error("$CLIENT_SECRET_JSON not defined.");
  }
  char resolved_path[PATH_MAX];
  char * found = realpath(client_json_path, resolved_path);
  if (found == NULL) {
    throw std::runtime_error("Client secret JSON not found at specified path.");
  }
  std::string real_path(resolved_path);

  std::string combined_command = OAUTH2_CLI_EXE + " " + resolved_path;
  std::cout << "oauth2_cli tool specified: " << combined_command << std::endl;
  std::cout << std::endl;

  // Executable oauth2_cli application with passed in CLIENT_SECRET_JSON.
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

  // Extract OAuth2 JSON token from oauth2_cli application output.
  std::regex oauth2_token_regex(
      "(.|\n)+CREDENTIALS_START\n(.+)\nCREDENTIALS_END", std::regex::extended);
  std::smatch matches;

  if (std::regex_search(result, matches, oauth2_token_regex) == 0) {
    throw std::runtime_error("No OAuth2 token found.");
  }

  // Convert OAuth2 JSON string to JSON object.
  std::string cleaned_str(matches[2].str());
  replace_all(cleaned_str, "'", "\"");
  return nlohmann::json::parse(cleaned_str);
}

int main(int argc, char** argv) {
  std::unique_ptr<corenode::SslKeyCert> sslKeyCert;
  std::shared_ptr<grpc::ChannelCredentials> credentials;
  std::string target_str(get_target(argc, argv));
  nlohmann::json oauth2_credential;

  if (OAUTH2_CLI_EXE.compare("NULL") != 0) {
    oauth2_credential = request_oauth_credential();
    std::cout << "stored_credential: " << oauth2_credential << std::endl;
  }

  try {
    sslKeyCert = std::unique_ptr<corenode::SslKeyCert>(new corenode::SslKeyCert);
    std::shared_ptr<grpc::ChannelCredentials> tlsCredentials =
      sslKeyCert->GenerateChannelCredentials();
    credentials = grpc::CompositeChannelCredentials(tlsCredentials,
        std::shared_ptr<grpc::CallCredentials>(
          grpc::AccessTokenCredentials(oauth2_credential["token"])));
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

#include "oauth2_token_processor.h"

corenode::OAuth2TokenProcessor::OAuth2TokenProcessor(
    std::shared_ptr<corenode::SslKeyCert> ssl_key_cert)
    : ssl_key_cert(ssl_key_cert) {}

grpc::Status corenode::OAuth2TokenProcessor::Process(
    const InputMetadata& auth_metadata,
    grpc::AuthContext* context,
    OutputMetadata* consumed_auth_metadata,
    OutputMetadata* response_metadata) {
  // DEBUG_INFO_START
  std::cout << "Using token processor..." << std::endl;
  std::multimap<grpc::string_ref, grpc::string_ref>::const_iterator itr;
  for (itr = auth_metadata.begin(); itr != auth_metadata.end(); ++itr) {
    std::cout << itr->first << "," << itr->second << std::endl;
  }
  // DEBUG_INFO_END

  // Determine intercepted method
  std::multimap<grpc::string_ref, grpc::string_ref>::const_iterator path =
    auth_metadata.find(":path");
  if (path == auth_metadata.end()) {
    return grpc::Status(grpc::StatusCode::INTERNAL, "Unknown path.");
  }

  // Verify request contains access token.
  std::multimap<grpc::string_ref, grpc::string_ref>::const_iterator token =
    auth_metadata.find("authorization");
  if (token == auth_metadata.end()) {
    return grpc::Status(grpc::StatusCode::UNAUTHENTICATED, "Missing access token.");
  }

  std::string raw(token->second.data());
  std::string bearer_token(raw.substr(
        BEARER_TEXT_LENGTH, token->second.length() - BEARER_TEXT_LENGTH));
  get_token_info(bearer_token);

  // return grpc::Status(grpc::StatusCode::UNAUTHENTICATED, "Missing token.");
  return grpc::Status::OK;
}

/**
 * Suppresses the curl output when used as the WriteFunction callback.
 */
size_t curl_write_function(void* buffer, size_t size, size_t nmemb) {
  return size * nmemb;
}

nlohmann::json corenode::OAuth2TokenProcessor::get_token_info(
    const std::string& token) {
  size_t url_length = token.length() + TOKEN_INFO_ENDPOINT.length();
  char buffer[url_length];
  snprintf(buffer, url_length, TOKEN_INFO_ENDPOINT.c_str(), token.c_str());

  curlpp::Cleanup cleanup;
  curlpp::Easy request;
  request.setOpt<curlpp::options::Url>(std::string(buffer));
  request.setOpt<curlpp::options::WriteFunction>(curl_write_function);
  request.perform();

  long http_code = curlpp::Infos::ResponseCode::get(request);

  std::cout << "HTTP CODE: " << http_code << std::endl;

  return nlohmann::json();
}

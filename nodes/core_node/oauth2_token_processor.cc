#include "oauth2_token_processor.h"

grpc::Status corenode::OAuth2TokenProcessor::Process(const InputMetadata &auth_metadata,
    grpc::AuthContext *context, OutputMetadata *consumed_auth_metadata,
    OutputMetadata *response_metadata) {
  std::cout << "Using token processor..." << std::endl;
  InputMetadata copy(auth_metadata);
  std::multimap<grpc::string_ref, grpc::string_ref>::iterator itr;
  for (itr = copy.begin(); itr != copy.end(); ++itr) {
    std::cout << itr->first << "," << itr->second << std::endl;
  }
  return grpc::Status(grpc::StatusCode::UNAUTHENTICATED, "Missing token.");
}

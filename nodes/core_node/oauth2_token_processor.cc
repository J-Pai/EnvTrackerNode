#include "oauth2_token_processor.h"

grpc::Status corenode::OAuth2TokenProcessor::Process(const InputMetadata &auth_metadata,
    grpc::AuthContext *context, OutputMetadata *consumed_auth_metadata,
    OutputMetadata *response_metadata) {
  std::cout << "Using token processor..." << std::endl;
  return grpc::Status::OK;
}

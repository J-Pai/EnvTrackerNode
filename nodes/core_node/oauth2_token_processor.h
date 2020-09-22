#ifndef ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_
#define ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_

#include <grpcpp/grpcpp.h>

namespace corenode {
class OAuth2TokenProcessor final : public grpc::AuthMetadataProcessor {
  public:
    grpc::Status Process(const InputMetadata&, grpc::AuthContext*,
        OutputMetadata*, OutputMetadata*) override;
  private:
    std::map<std::string, std::string> tokens;
};
}

#endif // ENVTRACKERNODE_CORENODE_OAUTH2_TOKEN_PROCESSOR_H_

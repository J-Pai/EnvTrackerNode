#ifndef ENVTRACKERNODE_CORENODE_SSL_KEY_CERT_H_
#define ENVTRACKERNODE_CORENODE_SSL_KEY_CERT_H_

#include <fstream>
#include <iostream>
#include <linux/limits.h>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>

#include <grpcpp/grpcpp.h>

namespace corenode {
class SslKeyCert final {
  public:
    SslKeyCert();
    SslKeyCert(const std::string&,
        const std::string&, const std::string&);

    /**
     * Generates an {@link grpc::ServerCredentials} object.
     * Uses the stored SSL key, certificate, and root CA.
     */
    std::shared_ptr<grpc::ServerCredentials> GenerateServerCredentials();

    /**
     * Generates an {@link grpc::ServerCredentials} object.
     * Uses the stored SSL key, certificate, and root CA.
     */
    std::shared_ptr<grpc::ChannelCredentials> GenerateChannelCredentials();
    std::string getKey();
    std::string getCert();
    std::string getRoot();
  private:
    std::string key;
    std::string cert;
    std::string root;

    void ReadFile(const std::string&, std::string&);
};
} // namespace corenode

#endif // ENVTRACKERNODE_CORENODE_SSL_KEY_CERT_H_

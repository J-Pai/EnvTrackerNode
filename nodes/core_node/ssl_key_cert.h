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
    std::shared_ptr<grpc::ServerCredentials> GenerateServerCredentials();
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
}

#endif // ENVTRACKERNODE_CORENODE_SSL_KEY_CERT_H_

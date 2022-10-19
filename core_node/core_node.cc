#include <iostream>
#include <memory>
#include <string>

#include <glog/logging.h>
#include <grpcpp/completion_queue.h>
#include <grpcpp/grpcpp.h>
#include <grpcpp/support/server_callback.h>

#include "proto/core_node.grpc.pb.h"

namespace envtrackernode::corenode {
  class ServerImpl final {
   public:
    ~ServerImpl(void) {
      server_->Shutdown();
      cq_->Shutdown();
    }

    void Run(const std::string server_address) {
      grpc::ServerBuilder builder;
      builder.AddListeningPort(server_address,
                               grpc::InsecureServerCredentials());
      builder.RegisterService(&service_);
      cq_ = builder.AddCompletionQueue();
      server_ = builder.BuildAndStart();

      LOG(INFO) << "Server listening on " << server_address;

      HandleRpcs();
    }

   private:
    class CallData {
     public:
      CallData(CoreNode::AsyncService* service,
               grpc::ServerCompletionQueue* cq)
        : service_(service), cq_(cq), responder_(&ctx_), status_(CREATE) {
        Proceed();
      }

      void Proceed(void) {
        switch(status_) {
          case CREATE: {
            status_ = PROCESS;
            service_->RequestSayHello(
                &ctx_, &request_, &responder_, cq_, cq_, this);
            break;
          }
          case PROCESS: {
            new CallData(service_, cq_);
            std::string prefix("Hello ");
            response_.set_message(prefix + request_.name());
            status_ = FINISH;
            responder_.Finish(response_, grpc::Status::OK, this);
            break;
          }
          default: {
            GPR_ASSERT(status_ == FINISH);
            delete this;
          }
        }
      }

     private:
      CoreNode::AsyncService* service_;
      grpc::ServerCompletionQueue* cq_;
      grpc::ServerContext ctx_;

      HelloRequest request_;
      HelloReply response_;

      grpc::ServerAsyncResponseWriter<HelloReply> responder_;

      enum CallStatus { CREATE, PROCESS, FINISH };
      CallStatus status_;
    };

    void HandleRpcs(void) {
      new CallData(&service_, cq_.get());
      void *tag;
      bool ok;

      while(true) {
        GPR_ASSERT(cq_->Next(&tag, &ok));
        GPR_ASSERT(ok);
        static_cast<CallData *>(tag)->Proceed();
      }
    }

    std::unique_ptr<grpc::ServerCompletionQueue> cq_;
    CoreNode::AsyncService service_;
    std::unique_ptr<grpc::Server> server_;
  };
}  // envtrackernode::corenode

int main(int argc, char** argv) {
  google::InitGoogleLogging(argv[0]);
  envtrackernode::corenode::ServerImpl server;
  server.Run("0.0.0.0:50051");

  return 0;
}

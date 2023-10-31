use tonic::{transport::Server, Request, Response, Status};

use envtrackernode::control_node_server::{ControlNode, ControlNodeServer};
use envtrackernode::{EchoReply, EchoRequest};

mod envtrackernode {
    tonic::include_proto!("envtrackernode");
}

#[derive(Debug, Default)]
struct ControlNodeRpc {}

#[tonic::async_trait]
impl ControlNode for ControlNodeRpc {
    async fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = EchoReply {
            message: format!("Rust Hello {}!", request.into_inner().message).into(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "localhost:58051".parse()?;

    let control_node = ControlNodeRpc::default();

    Server::builder()
        .add_service(ControlNodeServer::new(control_node))
        .serve(addr)
        .await?;

    Ok(())
}

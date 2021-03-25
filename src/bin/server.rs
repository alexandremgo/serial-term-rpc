use tonic::{transport::Server, Request, Response, Status};

use serial_term_rpc::serial_port::SerialPort;

// Brings into scope the module created by tonic.
pub mod serial_terminal {
    tonic::include_proto!("serial_terminal");
}

// Created when building the proto with tonic.
use serial_terminal::serial_com_service_server::{SerialComService, SerialComServiceServer};
use serial_terminal::{SerialPingReq, SerialPingRep, 
    PortListReq, PortListRep, 
    OpenPortReq, OpenPortRep,
    ClosePortReq, ClosePortRep,
    SendOnceReq, SendOnceRep,
    ReadOnceReq, ReadOnceRep
};

use std::sync::{Arc, Mutex};

pub struct MySerialComService {
    // Arc and Mutex to be able to safely share the port accross threads.
    port: Arc<Mutex<SerialPort>>,
}

#[tonic::async_trait]
impl SerialComService for MySerialComService {

    async fn ping(
            &self,
            _request: Request<SerialPingReq>,
        ) -> Result<Response<SerialPingRep>, Status> {

            println!("Got a ping request.");

            let reply = SerialPingRep {
                content: format!("Pong!").into(),
            };

            Ok(Response::new(reply))
    }

    async fn get_port_list(
            &self,
            _request: Request<PortListReq>,
        ) -> Result<Response<PortListRep>, Status> {

            println!("Got a GetPortList request.");

            let port_names = SerialPort::get_available_port_names();

            let reply = PortListRep {
                ports: port_names,
            };

            Ok(Response::new(reply))
    }

    async fn open_port(
            &self,
            request: Request<OpenPortReq>,
        ) -> Result<Response<OpenPortRep>, Status> {

            println!("Got a OpenPort request.");

            let request = request.into_inner();

            let port = Arc::clone(&self.port);
            let mut guard_port = port.lock().unwrap();
            let unlocked_port = &mut *guard_port;

            let resp = unlocked_port.open_port(&request.port, request.baudrate);

            let reply = OpenPortRep {
                success: resp.success,
                content: resp.content.into(),
            };

            Ok(Response::new(reply))
    }

    async fn close_port(
            &self,
            _request: Request<ClosePortReq>,
        ) -> Result<Response<ClosePortRep>, Status> {

            println!("Got a ClosePort request.");

            let port = Arc::clone(&self.port);
            let mut guard_port = port.lock().unwrap();
            let unlocked_port = &mut *guard_port;

            let resp = unlocked_port.close_port();

            let reply = ClosePortRep {
                success: resp.success,
                content: resp.content.into(),
            };

            Ok(Response::new(reply))
    }

    async fn send_once(
            &self,
            request: Request<SendOnceReq>,
        ) -> Result<Response<SendOnceRep>, Status> {

            println!("Got a SendOnce request.");

            let request = request.into_inner();

            let port = Arc::clone(&self.port);
            let mut guard_port = port.lock().unwrap();
            let unlocked_port = &mut *guard_port;

            let resp = unlocked_port.send_once(&request.content);

            let reply = SendOnceRep {
                success: resp.success,
                content: resp.content.into(),
            };

            Ok(Response::new(reply))
    }

    async fn read_once(
            &self,
            _request: Request<ReadOnceReq>,
        ) -> Result<Response<ReadOnceRep>, Status> {

            println!("Got a ReadOnce request.");

            let port = Arc::clone(&self.port);
            let mut guard_port = port.lock().unwrap();
            let unlocked_port = &mut *guard_port;

            let resp = unlocked_port.read_once();

            let reply = ReadOnceRep {
                success: resp.success,
                content: resp.content.into(),
            };

            Ok(Response::new(reply))
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:3333".parse()?;

    let port = Arc::new(Mutex::new(SerialPort::new()));
    let serial_com_service = MySerialComService { port: port };

    println!("Running the RPC server ...");

    Server::builder()
        .add_service(SerialComServiceServer::new(serial_com_service))
        .serve(addr)
        .await?;

    Ok(())
}
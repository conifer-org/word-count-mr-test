use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use tonic::codegen::ok;

pub mod worker {
    tonic::include_proto!("worker");
}

use worker::worker_server::{Worker, WorkerServer};
use worker::{MapReq, MapRes, RedReq, RedRes, KeyVal};

#[derive(Debug, Default)]
pub struct WorkerService {}

#[tonic::async_trait]
impl Worker for WorkerService{

    async fn mapper(&self, request: Request<MapReq>) -> Result<Response<MapRes>, Status>{
        println!("Got request");
        let req = request.into_inner();

        let mut mapper_vec: Vec<KeyVal> = vec![];

        for word in req.data.split_whitespace(){
            mapper_vec.push(KeyVal{
                key: word.to_string(),
                val: 1
            });
        }

        let res = MapRes {
            list_key_val: mapper_vec
        };

        Ok(Response::new(res))
    }

    async fn reducer(&self, request: Request<RedReq>) -> Result<Response<RedRes>, Status>{
        println!("Got request");
        let req = request.into_inner();

        let dictionary = Arc::new(Mutex::new(HashMap::new()));
        let dict_ref = Arc::clone(&dictionary);
        let mut wc_dict = dict_ref.lock().unwrap();

        for key_val in req.list_key_val{
            if !wc_dict.contains_key(key_val.key.as_str()) {
                wc_dict.insert(key_val.key.to_string(), key_val.val);
            } else {
                *wc_dict.get_mut(&key_val.key).unwrap() += key_val.val;
            }
        }

        // for word in req.data.split_whitespace(){
        //     if !wc_dict.contains_key(word) {
        //         wc_dict.insert(word.to_string(), 1);
        //     } else {
        //         *wc_dict.get_mut(word).unwrap() += 1;
        //     }
        // }

        let res = RedRes {
            wc_map: wc_dict.to_owned()
        };

        Ok(Response::new(res))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "[::1]:10001".parse().unwrap();
    let worker_service = WorkerService::default();

    Server::builder()
        .add_service(WorkerServer::new(worker_service))
        .serve(address)
        .await?;

    Ok(())
}
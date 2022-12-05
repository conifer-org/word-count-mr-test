use std::cmp::{max, min};
use std::collections::HashMap;
use std::ptr::null;
use std::sync::Arc;
use std::sync::Mutex;
use rand::seq::SliceRandom;
// use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
use tonic::codegen::{Body, ok};
use tonic::codegen::futures_core::Stream;

pub mod master {
    tonic::include_proto!("master");
}

pub mod worker {
    tonic::include_proto!("worker");
}

use master::master_server::{Master, MasterServer};
use master::{WriteReq, WriteRes};

use worker::worker_client::WorkerClient;
use worker::{MapReq, RedReq, KeyVal};

#[derive(Debug, Default)]
pub struct MasterService {}

const blocksize: usize = 20;
const workers: [&str; 1] = ["http://[::1]:10001"];
// const workers: Vec<&str> = vec!["http://[::1]:10001", "http://[::1]:10002", "http://[::1]:10003"];
const reducers: usize = 3;

#[tonic::async_trait]
impl Master for MasterService{

    // async fn write(&self, request: Request<WriteReq>) -> Result<Response<WriteRes>, Status>{
    //
    //     println!("Got request");
    //
    //     // let dictionary = Arc::new(Mutex::new(HashMap::new()));
    //     // let dict_ref = Arc::clone(&dictionary);
    //     // let mut wc_dict = dict_ref.lock().unwrap();
    //
    //     let mut dict_vec: Vec<HashMap<String,u64>> = vec![];
    //
    //     let req = request.into_inner();
    //
    //     for i in (0..req.data.len()).step_by(blocksize) {
    //         let sub_text = &req.data[i..min(i+blocksize, req.data.len())];
    //
    //         let rand_worker = workers.choose(&mut rand::thread_rng()).unwrap().clone();
    //         let mut client = WorkerClient::connect(rand_worker).await.unwrap();
    //         let request = tonic::Request::new(
    //             MapReq{
    //                 data: sub_text.to_owned()
    //             }
    //         );
    //         let response = client.mapper(request).await.unwrap();
    //
    //         dict_vec.push(response.into_inner().wc_map);
    //
    //     }
    //     // let mut wc_dict :HashMap<String,u64> = HashMap::new();
    //     let mut wc_dict1 :List<String,u64> = HashMap::new();
    //     let mut wc_dict2 :HashMap<String,u64> = HashMap::new();
    //
    //     println!("{}", xxx[0]);
    //     for i in dict_vec {
    //         println!("{:?}", i);
    //         for (key, val) in i {
    //             if key.as_bytes()[0] <= "m".as_bytes()[0]{
    //                 println!("less");
    //             }
    //             else{
    //                 println!("more");
    //             }
    //             if !wc_dict.contains_key(&key) {
    //                 wc_dict.insert(key.to_string(), val);
    //             } else {
    //                 *wc_dict.get_mut(&key).unwrap() += val;
    //             }
    //             // print!("{} {}",key, val);
    //         }
    //     }
    //
    //     let res = WriteRes {
    //         wc_map: wc_dict
    //     };
    //     Ok(Response::new(res))
    // }

    async fn word_count(&self, request: Request<WriteReq>) -> Result<Response<WriteRes>, Status>{

        println!("Got request");

        // let dictionary = Arc::new(Mutex::new(HashMap::new()));
        // let dict_ref = Arc::clone(&dictionary);
        // let mut wc_dict = dict_ref.lock().unwrap();

        let mut list_key_val_vec: Vec<Vec<KeyVal>> = vec![];

        let req = request.into_inner();

        for i in (0..req.data.len()).step_by(blocksize) {
            let sub_text = &req.data[i..min(i+blocksize, req.data.len())];

            let rand_worker = workers.choose(&mut rand::thread_rng()).unwrap().clone();
            let mut client = WorkerClient::connect(rand_worker).await.unwrap();
            let request = tonic::Request::new(
                MapReq{
                    data: sub_text.to_owned()
                }
            );
            let response = client.mapper(request).await.unwrap();
            list_key_val_vec.push(response.into_inner().list_key_val);

        }
        // let mut wc_dict :HashMap<String,u64> = HashMap::new();
        let mut mapper_vec1: Vec<KeyVal> = vec![];
        let mut mapper_vec2: Vec<KeyVal> = vec![];

        for i in list_key_val_vec {
            for key_val in i {
                if key_val.key.as_bytes()[0] <= "m".as_bytes()[0]{
                    // println!("less");
                    mapper_vec1.push(key_val);
                }
                else{
                    // println!("more");
                    mapper_vec2.push(key_val);
                }
            }
        }

        let rand_worker = workers.choose(&mut rand::thread_rng()).unwrap().clone();
        let mut client = WorkerClient::connect(rand_worker).await.unwrap();
        let request = tonic::Request::new(
            RedReq{
                list_key_val: mapper_vec1
            }
        );
        let response1 = client.reducer(request).await.unwrap();

        let rand_worker = workers.choose(&mut rand::thread_rng()).unwrap().clone();
        let mut client = WorkerClient::connect(rand_worker).await.unwrap();
        let request = tonic::Request::new(
            RedReq{
                list_key_val: mapper_vec2
            }
        );
        let response2 = client.reducer(request).await.unwrap();

        // println!("{:?}", response1.into_inner().wc_map);
        // println!("{:?}", response2.into_inner().wc_map);

        let mut wc_dict :HashMap<String,u64> = HashMap::new();

        for (key, val) in response1.into_inner().wc_map {
            if !wc_dict.contains_key(&key) {
                wc_dict.insert(key.to_string(), val);
            } else {
                *wc_dict.get_mut(&key).unwrap() += val;
            }
            // print!("{} {}",key, val);
        }
        for (key, val) in response2.into_inner().wc_map {
            if !wc_dict.contains_key(&key) {
                wc_dict.insert(key.to_string(), val);
            } else {
                *wc_dict.get_mut(&key).unwrap() += val;
            }
            // print!("{} {}",key, val);
        }

        let res = WriteRes {
            wc_map: wc_dict
        };
        Ok(Response::new(res))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "[::1]:8080".parse().unwrap();
    let master_service = MasterService::default();

    Server::builder()
        .add_service(MasterServer::new(master_service))
        .serve(address)
        .await?;

    Ok(())
}
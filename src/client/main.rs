use master::master_client::MasterClient;
use master::WriteReq;

use worker::worker_client::WorkerClient;

pub mod master {
    tonic::include_proto!("master");
}

pub mod worker {
    tonic::include_proto!("worker");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MasterClient::connect("http://[::1]:8080").await?;
    let request = tonic::Request::new(
        WriteReq{
            data: "Oceans and lakes have much in common, but they are also quite different.\
                   Both are bodies of water, but oceans are very large bodies of salt water,\
                   while lakes are much smaller bodies of fresh water. Lakes are usually surrounded \
                   by land, while oceans are what surround continents. Both have plants and animals \
                   living in them. The ocean is home to the largest animals on the planet, whereas \
                   lakes support much smaller forms of life. When it is time for a vacation, both \
                   will make a great place to visit and enjoy".to_owned()
        }
    );
    let response = client.word_count(request).await?;
    println!("RESPONSE {:?}", response.into_inner().wc_map);

    // let mut client = WorkerClient::connect("http://[::1]:10001").await?;
    // let request = tonic::Request::new(
    //     WcReq{
    //         data: "Hello world how are doing world".to_owned()
    //     }
    // );
    // let response = client.word_count(request).await?;
    // println!("RESPONSE {:?}", response.into_inner().wc_map);

    Ok(())
}
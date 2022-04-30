use aws_sdk_s3::{types::ByteStream, Client, Error, Region, PKG_VERSION};
use std::path::Path;
use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    pub region: Option<String>,

    /// The name of the bucket.
    #[structopt(short, long)]
    pub bucket: String,

    /// The name of the file to upload.
    #[structopt(short, long)]
    pub filename: String,

    /// The name of the object in the bucket.
    #[structopt(short, long)]
    pub key: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    pub verbose: bool,
}

pub async fn download_bytes(client: &Client, bucket: &str, key: &str) -> Result<(), Error> {
    println!("will get object..");

    let resp = client.get_object().bucket(bucket).key(key).send().await?;
    let data = resp.body.collect().await;

    let bytes = data.unwrap().into_bytes().to_vec();
    println!("data: {:?}", bytes);

    Ok(())
}

// Upload bytes to a bucket.
// snippet-start:[s3.rust.s3-helloworld]
pub async fn upload_bytes(
    client: &Client,
    bucket: &str,
    bytes: Vec<u8>,
    key: &str,
) -> Result<(), Error> {
    // let stream = ByteStream::from(bytes);
    let stream = bytes.into();
    println!("will put object..");

    let resp = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(stream)
        .send()
        .await?;

    println!("Upload success. Version: {:?}", resp.version_id);

    let resp = client.get_object().bucket(bucket).key(key).send().await?;
    let data = resp.body.collect().await;
    println!("data: {:?}", data.unwrap().into_bytes());

    Ok(())
}

// Upload a file to a bucket.
// snippet-start:[s3.rust.s3-helloworld]
pub async fn upload_object(
    client: &Client,
    bucket: &str,
    filename: &str,
    key: &str,
) -> Result<(), Error> {
    let body = ByteStream::from_path(Path::new(filename)).await;

    match body {
        Ok(b) => {
            println!("will put object..");

            let resp = client
                .put_object()
                .bucket(bucket)
                .key(key)
                .body(b)
                .send()
                .await?;

            println!("Upload success. Version: {:?}", resp.version_id);

            let resp = client.get_object().bucket(bucket).key(key).send().await?;
            let data = resp.body.collect().await;
            println!("data: {:?}", data.unwrap().into_bytes());
        }
        Err(e) => {
            println!("Got an error uploading object:");
            println!("{}", e);
            process::exit(1);
        }
    }

    Ok(())
}

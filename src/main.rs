#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use crate::dao::aws::{download_bytes, upload_bytes, upload_object, Opt};
use crate::dao::image_dao::MemImageDaoImpl;
use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{types::ByteStream, Client, Error, Region, PKG_VERSION};
use dao::image_dao::ImageDao;
use dotenv::dotenv;
use rocket::data::ToByteUnit;
use rocket::State;
use rocket::{response::Debug, Data};
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use structopt::StructOpt;

mod dao;
mod logger;

#[post("/image", format = "binary", data = "<data>")]
async fn save_image(
    dao: &State<Box<dyn ImageDao>>,
    data: Data<'_>,
) -> Result<String, rocket::response::Debug<anyhow::Error>> {
    let mut vec = vec![];
    let byte_count = data
        .open(2.mebibytes())
        .stream_to(&mut vec)
        .await
        .map_err(|e| Debug(anyhow::Error::msg(format!("{e:?}"))))?;

    println!("byte_count: {:?}", byte_count);
    println!("vec: {:?}", vec);

    dao.save_image(&vec)?;

    Ok("done...".to_owned())
}

#[get("/image/<id>", format = "binary")]
fn get_image(
    dao: &State<Box<dyn ImageDao>>,
    id: String,
) -> Result<Option<Vec<u8>>, Debug<anyhow::Error>> {
    Ok(dao.load_image(&id)?)
}

#[test]
fn write_bytes_to_file() {
    let bytes = &[1, 2, 3, 10, 200];
    std::fs::write(&format!("./tmp_bytes"), bytes).unwrap();
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     // init_logger();

//     // let image_dao: Box<dyn ImageDao> = Box::new(ImageDaoImpl {
//     let image_dao: Box<dyn ImageDao> = Box::new(MemImageDaoImpl {
//         state: Mutex::new(HashMap::new()),
//     });
//     image_dao.init()?;

//     rocket::build()
//         .manage(image_dao)
//         .mount("/", routes![get_image, save_image])
//         // .register("/", catchers![not_found])
//         .launch()
//         .await?;

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    // dotenv().ok();

    let Opt {
        bucket,
        filename,
        key,
        region,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    println!();

    if verbose {
        println!("S3 client version: {}", PKG_VERSION);
        println!(
            "Region:            {}",
            region_provider.region().await.unwrap().as_ref()
        );
        println!("Bucket:            {}", &bucket);
        println!("Filename:          {}", &filename);
        println!("Key:               {}", &key);
        println!();
    }

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    // let bytes = vec![1, 2, 3, 4, 5, 6, 7, 8, 10, 200];
    // let res = upload_bytes(&client, &bucket, bytes, &key).await?;
    // // let res = upload_object(&client, &bucket, &filename, &key).await?;
    // println!("upload res: {:?}", res);

    let res = download_bytes(&client, &bucket, &key).await?;
    println!("download res: {:?}", res);

    Ok(())
}

fn frontend_host(env: &Env) -> &'static str {
    match env {
        Env::Local => "http://localhost:3000",
        Env::Test => "http://foo.capi.finance",
    }
}

#[derive(Debug, Clone)]
pub enum Env {
    Local,
    Test,
}

fn environment() -> Env {
    dotenv().ok();
    let env = env::var("TEST_ENV").unwrap();
    println!("Env value: {}", env);
    let env = if env == "1" { Env::Test } else { Env::Local };
    log::info!("Environment: {:?}", env);
    env
}

#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use anyhow::Result;
use dao::image_dao::{AwsImageDao, ImageDao};
use dotenv::dotenv;
use rocket::data::ToByteUnit;
use rocket::State;
use rocket::{response::Debug, Data};
use std::env;

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

    dao.save_image(vec).await?;

    Ok("done...".to_owned())
}

#[get("/image/<id>", format = "binary")]
async fn get_image(
    dao: &State<Box<dyn ImageDao>>,
    id: String,
) -> Result<Option<Vec<u8>>, Debug<anyhow::Error>> {
    Ok(dao.load_image(&id).await?)
}

#[test]
fn write_bytes_to_file() {
    let bytes = &[1, 2, 3, 10, 200];
    std::fs::write(&format!("./tmp_bytes"), bytes).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    // init_logger();

    let image_dao: Box<dyn ImageDao> = Box::new(AwsImageDao::new().await?);

    rocket::build()
        .manage(image_dao)
        .mount("/", routes![get_image, save_image])
        // .register("/", catchers![not_found])
        .launch()
        .await?;

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

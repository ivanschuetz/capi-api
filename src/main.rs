#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use crate::dao::db::create_db_client;
use crate::dao::image_dao::MemImageDaoImpl;
use anyhow::{anyhow, Result};
use dao::image_dao::{ImageDao, ImageDaoImpl};
use dotenv::dotenv;
use logger::init_logger;
use rocket::data::ToByteUnit;
use rocket::http::hyper::StatusCode;
use rocket::http::Status;
use rocket::response::status::NotFound;
use rocket::{response::Debug, Data, Response};
use rocket::{Request, State};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use std::{convert, env, fs, io};

mod dao;
mod logger;

// type DbConn = Mutex<Connection>;
// type MyImageDao = Mutex<ImageDao>;

// #[get("/")]
// fn hello(db_conn: State<DbConn>) -> Result<String, Debug<Error>> {
//     db_conn
//         .lock()
//         .expect("db connection lock")
//         .query_row("SELECT name FROM entries WHERE id = 0", &[], |row| {
//             row.get(0)
//         })
//         .map_err(Debug)
// }

// #[get("/")]
// // fn hello(dao: State<DbConn>) -> Result<String, Debug<Error>> {
// fn hello(dao: State<Mutex<Box<dyn ImageDao>>>) -> Result<String, Debug<Error>> {
//     // fn hello(dao: State<Mutex<ImageDaoImpl>>) -> Result<String, Debug<Error>> {
//     Ok("great!".to_owned())
// }

// #[get("/hello/<name>/<age>")]
// fn hello(dao: State<Box<dyn ImageDao>>, name: String, age: u8) -> String {
//     println!("in hello...");
//     println!("dao: {:?}", dao.load_image("iaaaad"));

//     format!("Hello, {} year old named {}!", age, name)
// }

// #[get("/hello/<name>/<age>")]
// fn save_image(name: String, age: u8) -> String {
//     format!("Hello, {} year old named {}!", age, name)
// }

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

// #[get("/image/<id>", format = "binary")]
// fn get_image(dao: State<Mutex<Box<dyn ImageDao>>>, id: String) -> Result<Vec<u8>, (Status, Error)> {
//     println!("get image: {:?}", id);

//     let dao = dao
//         .lock()
//         .map_err(|e| (Status::InternalServerError, "...".to_owned()))?;

//     let bytes = dao
//         .load_image(&id)
//         .map_err(|e| (Status::InternalServerError, "...".to_owned()))?;
//     println!("loaded bytes: {:?}", bytes);

//     match bytes {
//         Some(bytes) => Ok(bytes),
//         None => Err((Status::NotFound, "...".to_owned())),
//     }
// }

#[catch(500)]
fn not_found(req: &Request) -> String {
    // req.
    format!("500 ---Sorry, '{}' is not a valid path.", req.uri())
}

#[get("/image/<id>", format = "binary")]
fn get_image(
    dao: &State<Box<dyn ImageDao>>,
    id: String,
) -> Result<Option<Vec<u8>>, Debug<anyhow::Error>> {
    Ok(dao.load_image(&id)?)
}

// #[get("/image/<id>", format = "binary")]
// fn get_image(
//     dao: &State<Box<dyn ImageDao>>,
//     id: String,
// ) -> Result<Vec<u8>, (Status, std::io::Error)> {
//     let bytes = dao.load_image(&id).map_err(|e| {
//         (
//             Status::InternalServerError,
//             std::io::Error::new(ErrorKind::Other, format!("oh no {e:?}!")),
//         )
//     })?;

//     match bytes {
//         Some(bytes) => Ok(bytes),
//         None => Err((
//             Status::NotFound,
//             std::io::Error::new(ErrorKind::Other, "not found!"),
//         )),
//     }
// }

// #[get("/image/<id>", format = "binary")]
// fn get_image(
//     dao: State<Mutex<Box<dyn ImageDao>>>,
//     id: String,
// ) -> Result<Vec<u8>, rocket::response::Debug<(Status, anyhow::Error)>> {
//     let dao = dao
//         .lock()
//         .map_err(|_| Debug((Status::InternalServerError, anyhow!("...".to_owned()))))?;

//     let bytes = dao
//         .load_image(&id)
//         .map_err(|_| Debug((Status::InternalServerError, anyhow!("...".to_owned()))))?;

//     match bytes {
//         Some(bytes) => Ok(bytes),
//         None => Err(Debug((Status::NotFound, anyhow!("...".to_owned())))),
//     }
// }

// #[get("/image/<id>", format = "binary")]
// fn get_image(dao: State<Mutex<Box<dyn ImageDao>>>, id: String) -> Result<Vec<u8>, Status> {
//     println!("get image: {:?}", id);

//     let dao = dao
//         .lock()
//         // .map_err(|e| Debug(Error::msg(format!("{e:?}"))))?;
//         .map_err(|e| {
//             // let msg =
//             // let msg = format!("...");
//             Status::new(
//                 StatusCode::InternalServerError.to_u16(),
//                 // msg.as_ref(), // format!("...").as_ref(),
//                 "...",
//             )
//         })?;

//     let bytes = dao
//         .load_image(&id)
//         .map_err(|e| Status::new(StatusCode::InternalServerError.to_u16(), "..."))?;
//     println!("loaded bytes: {:?}", bytes);

//     match bytes {
//         Some(bytes) => Ok(bytes),

//         // Ok(bytes),
//         None => Err(Status::new(StatusCode::NotFound.to_u16(), "...")),
//     }
// }

// pub type ErrorResponse = (Status, Error); // Or maybe (Status, Json<Error>)

// impl convert::From<Error> for ErrorResponse {
//     fn from(e: Error) -> ErrorResponse {
//         match e {
//             NotFoundError => (Status::NotFound, e),
//             OtherError => (Status::InternalServerError, e),
//         }
//     }
// }

#[test]
fn write_bytes_to_file() {
    let bytes = &[1, 2, 3, 10, 200];
    fs::write(&format!("./tmp_bytes"), bytes).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    // init_logger();

    // let conn = Connection::open_in_memory().expect("in memory db");

    // let db_client = Arc::new(create_db_client()?);
    // let image_dao: Box<dyn ImageDao> = Box::new(ImageDaoImpl {
    let image_dao: Box<dyn ImageDao> = Box::new(MemImageDaoImpl {
        state: Mutex::new(HashMap::new()), // client: db_client.clone(),
    });
    // let image_dao = ImageDaoImpl {
    //     // client: db_client.clone(),
    // };
    image_dao.init()?;

    println!("contune...");

    rocket::build()
        // .manage(Mutex::new(conn))
        // .manage(Mutex::new(image_dao))
        .manage(image_dao)
        // .mount("/", routes![hello])
        // .mount("/", routes![save_image, get_image])
        .mount("/", routes![get_image, save_image])
        // .register("/", catchers![not_found])
        // .catchers()
        .launch()
        .await?;

    Ok(())
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     init_logger();

//     let db_client = Arc::new(create_db_client().await?);
//     let image_dao: Arc<dyn ImageDao> = Arc::new(ImageDaoImpl {
//         client: db_client.clone(),
//     });
//     image_dao.init().await?;

//     let env = environment();

//     Ok(())
// }

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

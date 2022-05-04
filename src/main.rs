#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use algonaut::model::indexer::v2::QueryApplicationInfo;
use anyhow::Result;
use base::dependencies::{algod, indexer};
use base::flows::create_dao::setup_dao_specs::ImageHash;
use base::flows::create_dao::storage::load_dao::DaoAppId;
use base::state::dao_app_state::dao_global_state;
use dao::image_dao::{AwsImageDao, ImageDao};
use data_encoding::BASE64;
use dotenv::dotenv;
use rocket::data::ToByteUnit;
use rocket::http::Method;
use rocket::response::content::Custom;
use rocket::State;
use rocket::{response::Debug, Data};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use sha2::Digest;
use std::env;

mod dao;
mod logger;

#[post("/image/<app_id>", format = "binary", data = "<data>")]
async fn save_image(
    dao: &State<Box<dyn ImageDao>>,
    data: Data<'_>,
    app_id: u64,
) -> Result<Option<String>, rocket::response::Debug<anyhow::Error>> {
    let mut vec = vec![];

    // TODO reject if data > x size
    // the client has to compress / ask user to reduce

    let byte_count = data
        .open(2.mebibytes())
        .stream_to(&mut vec)
        .await
        .map_err(|e| Debug(anyhow::Error::msg(format!("{e:?}"))))?;

    println!("byte_count: {:?}", byte_count);
    println!("vec: {:?}", vec);

    let hash = hash(&vec);
    if is_on_chain_with_dao_state(app_id, &hash).await? {
        dao.save_image(&hash, vec).await?;
        Ok(Some("done...".to_owned()))
    } else {
        println!("Didn't find app id or hash in the app");
        Ok(None)
    }
}

// sends image as raw bytes - seems not needed. keeping it for now for historical / understanding purpose
#[get("/image/<id>", format = "binary")]
#[allow(dead_code)]
async fn get_image(
    dao: &State<Box<dyn ImageDao>>,
    id: String,
) -> Result<Option<Vec<u8>>, Debug<anyhow::Error>> {
    Ok(dao.load_image(&id).await?)
}

#[get("/image/<id>", format = "image/avif")]
async fn get_image_jpeg(
    dao: &State<Box<dyn ImageDao>>,
    id: String,
) -> Result<Custom<Option<Vec<u8>>>, Debug<anyhow::Error>> {
    let maybe_bytes = dao.load_image(&id).await?;
    // we expect all the stored images to be in jpeg format - if stored with our frontend (which should be the only one talking to the backend)
    // if somehow someone manages to store something not jpeg, then this response may be corrupted
    Ok(Custom(rocket::http::ContentType::JPEG, maybe_bytes))
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

    let env = environment();

    let frontend_host = frontend_host(&env);
    println!("frontend_host: {}", frontend_host);

    let allowed_origins = AllowedOrigins::some_exact(&[frontend_host]);

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        // allowed_headers: AllowedHeaders::some(&[
        //     "Authorization",
        //     "User-Agent",
        //     "Sec-Fetch-Mode",
        //     "Referer",
        //     "Origin",
        //     "Content-Type",
        //     "Accept",
        //     "Access-Control-Request-Method",
        //     "Access-Control-Request-Headers",
        // ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;

    rocket::build()
        .manage(image_dao)
        .mount("/", routes![get_image_jpeg, save_image])
        .attach(cors)
        // .register("/", catchers![not_found])
        .launch()
        .await?;

    Ok(())
}

fn hash(bytes: &[u8]) -> String {
    let hash = sha2::Sha512_256::digest(bytes);
    BASE64.encode(&hash)
}

async fn is_on_chain_with_dao_state(app_id: u64, hash: &str) -> Result<bool> {
    let algod = algod();

    let state = dao_global_state(&algod, DaoAppId(app_id)).await?;
    if state.image_hash == Some(ImageHash(hash.to_owned())) {
        return Ok(true);
    } else {
        return Ok(false);
    }
}

// we can also retrieve app state with the indexer, but
// 1) takes longer
// 2) we already have a state reader in the domain, which uses algod (used in is_on_chain_with_dao_state)
// leaving this anyway, since at least algoexplorer removed these kind of queries from algod (only possible with indexer)
#[allow(dead_code)]
async fn is_on_chain_indexer(app_id: u64, hash: &str) -> Result<bool> {
    let indexer = indexer();

    let app_info_res = indexer
        .application_info(
            app_id,
            &QueryApplicationInfo {
                include_all: Some(false),
            },
        )
        .await;

    match app_info_res {
        Ok(app_info) => match app_info.application {
            Some(app) => {
                let key_values = app.params.global_state;
                let key_value = key_values.into_iter().find(|kv| kv.key == "LogoUrl");
                match key_value {
                    Some(kv) => {
                        let bytes = kv.value.bytes;
                        Ok(String::from_utf8(bytes)? == hash)
                    }
                    // key not found in app - if it's a capi app, this should be rare, as dao setup initializes it (it can be empty but key should be there)
                    // can happen if: capi dao creation was interrupted (app created but not setup) or the app id is not a capi app
                    None => {
                        println!("App ({app_id}) key not present.");
                        Ok(false)
                    }
                }
            }
            // application info has no app - unclear when this happens (when the app doesn't exist, we get a 404)
            None => Ok(false),
        },
        Err(e) => {
            if e.is_404() {
                // application not found
                Ok(false)
            } else {
                Err(e)?
            }
        }
    }
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

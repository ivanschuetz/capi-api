#![feature(integer_atomics, proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use algonaut::model::indexer::v2::QueryApplicationInfo;
use anyhow::{Error, Result};
use dao::bytes_dao::{AwsBytesDao, BytesDao};
use data_encoding::BASE64;
use dotenv::dotenv;
use mbase::api::contract::Contract;
use mbase::api::teal_api::TealFileLoader;
use mbase::api::version::Version;
use mbase::dependencies::{algod, indexer};
use mbase::models::dao_app_id::DaoAppId;
use mbase::models::hash::GlobalStateHash;
use mbase::state::dao_app_state::dao_global_state;
use rocket::data::{ByteUnit, ToByteUnit};
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
    deps: &State<Box<Deps>>,
    data: Data<'_>,
    app_id: u64,
) -> Result<Option<String>, rocket::response::Debug<anyhow::Error>> {
    save_bytes(deps, data, app_id, 2.mebibytes(), StateField::ImageHash).await
}

#[post("/descr/<app_id>", format = "binary", data = "<data>")]
async fn save_descr(
    deps: &State<Box<Deps>>,
    data: Data<'_>,
    app_id: u64,
) -> Result<Option<String>, rocket::response::Debug<anyhow::Error>> {
    save_bytes(
        deps,
        data,
        app_id,
        500.kilobytes(),
        StateField::DescriptionHash,
    )
    .await
}

async fn save_bytes(
    deps: &State<Box<Deps>>,
    data: Data<'_>,
    app_id: u64,
    max_size: ByteUnit,
    state: StateField,
) -> Result<Option<String>, rocket::response::Debug<anyhow::Error>> {
    let mut vec = vec![];

    // TODO reject if data > x size
    // the client has to compress / ask user to reduce

    let byte_count = data
        // .open(500.kilobytes())
        .open(max_size)
        .stream_to(&mut vec)
        .await
        .map_err(|e| Debug(anyhow::Error::msg(format!("{e:?}"))))?;

    println!("byte_count: {:?}", byte_count);
    println!("vec: {:?}", vec);

    let hash = hash(&vec);
    if is_on_chain_with_dao_state(app_id, &hash, state).await? {
        deps.bytes_dao.save_bytes(&hash, vec).await?;
        Ok(Some("done...".to_owned()))
    } else {
        println!("Didn't find app id or hash in the app");
        Ok(None)
    }
}

#[get("/descr/<id>", format = "binary")]
#[allow(dead_code)]
async fn get_descr(
    deps: &State<Box<Deps>>,
    id: String,
) -> Result<Option<Vec<u8>>, Debug<anyhow::Error>> {
    Ok(deps.bytes_dao.load_bytes(&id).await?)
}

// TODO JSON - unclear how to use here. See https://rocket.rs/v0.5-rc/guide/responses/#json
#[get("/teal/versions", format = "json")]
#[allow(dead_code)]
async fn get_teal_versions(deps: &State<Box<Deps>>) -> Result<String, Debug<anyhow::Error>> {
    let versions = deps.teal_api.last_versions();
    let json = serde_json::to_string(&versions).map_err(Error::msg)?;
    Ok(json)
}

#[get("/teal/<contract>/<version>", format = "binary")]
#[allow(dead_code)]
async fn get_teal_template(
    deps: &State<Box<Deps>>,
    contract: String,
    version: String,
) -> Result<Option<Vec<u8>>, Debug<anyhow::Error>> {
    let contract = match contract.as_ref() {
        "approval" => Contract::DaoAppApproval,
        "clear" => Contract::DaoAppClear,
        "customer" => Contract::DaoCustomer,
        _ => return Ok(None),
    };

    let version_numer = version.parse().map_err(Error::msg)?;
    let version = Version(version_numer);

    let res = deps.teal_api.template(contract, version)?;
    Ok(res.map(|r| r.template.0))
}

#[get("/image/<id>", format = "image/avif")]
async fn get_image_jpeg(
    deps: &State<Box<Deps>>,
    id: String,
) -> Result<Custom<Option<Vec<u8>>>, Debug<anyhow::Error>> {
    let maybe_bytes = deps.bytes_dao.load_bytes(&id).await?;
    // we expect all the stored images to be in jpeg format - if stored with our frontend (which should be the only one talking to the backend)
    // if somehow someone manages to store something not jpeg, then this response may be corrupted
    Ok(Custom(rocket::http::ContentType::JPEG, maybe_bytes))
}

#[test]
fn write_bytes_to_file() {
    let bytes = &[1, 2, 3, 10, 200];
    std::fs::write(&format!("./tmp_bytes"), bytes).unwrap();
}

struct Deps {
    bytes_dao: Box<dyn BytesDao>,
    teal_api: TealFileLoader,
}

#[tokio::main]
async fn main() -> Result<()> {
    // init_logger();

    dotenv().ok();

    let bytes_dao: Box<dyn BytesDao> = Box::new(AwsBytesDao::new().await?);
    let teal_api = TealFileLoader {};

    let deps = Deps {
        bytes_dao,
        teal_api,
    };

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
        .manage(Box::new(deps))
        .mount(
            "/",
            routes![
                get_image_jpeg,
                save_image,
                get_descr,
                save_descr,
                get_teal_template,
                get_teal_versions
            ],
        )
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

#[derive(Debug, Clone, Copy)]
enum StateField {
    ImageHash,
    DescriptionHash,
}

async fn is_on_chain_with_dao_state(app_id: u64, hash: &str, field: StateField) -> Result<bool> {
    let algod = algod();

    let state = dao_global_state(&algod, DaoAppId(app_id)).await?;
    let value = match field {
        StateField::ImageHash => state.image_hash,
        StateField::DescriptionHash => state.project_desc,
    };

    if value == Some(GlobalStateHash(hash.to_owned())) {
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
async fn is_on_chain_indexer(app_id: u64, hash: &str, key: &str) -> Result<bool> {
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
                let key_value = key_values.into_iter().find(|kv| kv.key == key);
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
    let env = env::var("TEST_ENV").unwrap();
    println!("Env value: {}", env);
    let env = if env == "1" { Env::Test } else { Env::Local };
    log::info!("Environment: {:?}", env);
    env
}

#![feature(integer_atomics, proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use anyhow::{Error, Result};
use dotenv::dotenv;
use mbase::api::contract::Contract;
use mbase::api::teal_api::TealFileLoader;
use mbase::api::version::Version;
use mbase::dependencies::{self, Env};
use mbase::logger::init_logger;
use rocket::http::Method;
use rocket::response::Debug;
use rocket::State;
use rocket_cors::{AllowedHeaders, AllowedOrigins};

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

#[test]
fn write_bytes_to_file() {
    let bytes = &[1, 2, 3, 10, 200];
    std::fs::write(&format!("./tmp_bytes"), bytes).unwrap();
}

struct Deps {
    teal_api: TealFileLoader,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;

    dotenv().ok();

    log::debug!("---------------------------------------------");
    let env = dependencies::env();
    log::debug!("---------------------------------------------");

    let teal_api = TealFileLoader {};

    let deps = Deps { teal_api };

    let frontend_host = frontend_host(&env);
    log::info!("frontend_host: {frontend_host}");
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
        .mount("/", routes![get_teal_template, get_teal_versions])
        .attach(cors)
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

use aws_sdk_s3::error::GetObjectErrorKind;
use aws_sdk_s3::types::SdkError;
use aws_sdk_s3::{types::ByteStream, Client, Error};
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

pub async fn download_bytes(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<Option<Vec<u8>>, Error> {
    log::debug!("will get object..");

    match client.get_object().bucket(bucket).key(key).send().await {
        Ok(resp) => {
            let data = resp.body.collect().await;

            let bytes = data.unwrap().into_bytes().to_vec();
            log::debug!("data: {:?}", bytes);

            Ok(Some(bytes))
        }

        Err(e) => {
            log::error!("Error retrieving image: {} for key: {}", e, key);
            match &e {
                SdkError::ServiceError { err, .. } => match &err.kind {
                    GetObjectErrorKind::Unhandled(m) => {
                        let text = format!("{}", m);
                        // when downloading a not existing id, we get access denied, so map to None, so web framework returns 404
                        // this error isn't structured (just dyn error) so processing the text
                        if text.contains("code: \"AccessDenied\"") {
                            Ok(None)
                        } else {
                            Err(e.into())
                        }
                    }
                    // this is what we intuitively expect for "not found" but for some reason we get GetObjectErrorKind::Unhandled with "access denied"
                    // we handle it the same anyway
                    GetObjectErrorKind::NoSuchKey(m) => {
                        log::error!("No such key error: {} for key: {}", m, key);
                        Ok(None)
                    }
                    _ => Err(e.into()),
                },
                _ => Err(e.into()),
            }
        }
    }
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
    log::debug!("will put object..");

    let resp = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(stream)
        .send()
        .await?;

    log::debug!("Upload success. Version: {:?}", resp.version_id);

    let resp = client.get_object().bucket(bucket).key(key).send().await?;
    let data = resp.body.collect().await;
    log::trace!("data: {:?}", data.unwrap().into_bytes());

    Ok(())
}

// Upload a file to a bucket.
// snippet-start:[s3.rust.s3-helloworld]
#[allow(dead_code)]
pub async fn upload_object(
    client: &Client,
    bucket: &str,
    filename: &str,
    key: &str,
) -> Result<(), Error> {
    let body = ByteStream::from_path(Path::new(filename)).await;

    match body {
        Ok(b) => {
            log::debug!("will put object..");

            let resp = client
                .put_object()
                .bucket(bucket)
                .key(key)
                .body(b)
                .send()
                .await?;

            log::debug!("Upload success. Version: {:?}", resp.version_id);

            let resp = client.get_object().bucket(bucket).key(key).send().await?;
            let data = resp.body.collect().await;
            log::trace!("data: {:?}", data.unwrap().into_bytes());
        }
        Err(e) => {
            log::error!("Got an error uploading object:");
            log::error!("{}", e);
            process::exit(1);
        }
    }

    Ok(())
}

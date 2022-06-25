use crate::dao::aws::{download_bytes, upload_bytes};
use anyhow::Result;
use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Region, PKG_VERSION};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[async_trait]
pub trait BytesDao: Sync + Send {
    async fn save_bytes(&self, id: &str, bytes: Vec<u8>) -> Result<()>;
    async fn load_bytes(&self, id: &str) -> Result<Option<Vec<u8>>>;
}
pub struct AwsBytesDao {
    bucket: String,
    client: Arc<Client>,
}

impl AwsBytesDao {
    pub async fn new() -> Result<AwsBytesDao> {
        let bucket = "cimgbucket".to_owned();
        let region = "us-east-1";

        let region_provider = RegionProviderChain::first_try(Region::new(region))
            .or_default_provider()
            .or_else(Region::new("us-west-2"));

        log::info!("S3 client version: {}", PKG_VERSION);
        log::info!("Bucket: {}", &bucket);
        log::info!(
            "Region: {}",
            region_provider.region().await.unwrap().as_ref()
        );

        let shared_config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&shared_config);

        Ok(AwsBytesDao {
            bucket,
            client: Arc::new(client),
        })
    }
}

#[async_trait]
impl BytesDao for AwsBytesDao {
    async fn save_bytes(&self, id: &str, bytes: Vec<u8>) -> Result<()> {
        log::debug!("saving id: {:?} bytes: {:?}", id, bytes);
        let res = upload_bytes(&self.client, &self.bucket, bytes, id).await?;
        // let res = upload_object(&client, &bucket, &filename, &key).await?;
        log::debug!("upload res: {:?}", res);
        Ok(())
    }

    async fn load_bytes(&self, id: &str) -> Result<Option<Vec<u8>>> {
        log::debug!("loading bytes for id: {:?}", id);
        let res = download_bytes(&self.client, &self.bucket, id).await?;
        log::debug!("download res: {:?}", res);
        Ok(res)
    }
}

pub struct MemBytesDaoImpl {
    // TODO mutex needed?
    pub state: Mutex<HashMap<String, Vec<u8>>>,
}

#[async_trait]
impl BytesDao for MemBytesDaoImpl {
    async fn save_bytes(&self, id: &str, bytes: Vec<u8>) -> Result<()> {
        log::debug!("id: {:?}", id);

        let mut s = self.state.lock().unwrap();
        log::debug!("saving bytes: {:?}", bytes);
        s.insert(id.to_owned(), bytes);

        Ok(())
    }

    async fn load_bytes(&self, id: &str) -> Result<Option<Vec<u8>>> {
        log::debug!("loading bytes: {:?}", id);

        let s = self.state.lock().unwrap();
        Ok(s.get(id).map(|o| o.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, sync::Mutex};

    use super::{BytesDao, MemBytesDaoImpl};
    use crate::logger::init_logger;
    use anyhow::Result;

    #[test]
    #[ignore]
    fn test_init() -> Result<()> {
        init_logger();
        let bytes_dao = create_test_bytes_dao()?;
        Ok(())
    }

    // to be executed after test_init
    #[test]
    #[ignore]
    fn test_insert_and_load_an_image() -> Result<()> {
        init_logger();
        let _image_dao = create_test_bytes_dao()?;

        todo!();
    }

    fn create_test_bytes_dao() -> Result<Box<dyn BytesDao>> {
        Ok(Box::new(MemBytesDaoImpl {
            state: Mutex::new(HashMap::new()),
        }))
    }
}

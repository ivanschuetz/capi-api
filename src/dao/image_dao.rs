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
pub trait ImageDao: Sync + Send {
    async fn save_image(&self, id: &str, image: Vec<u8>) -> Result<()>;
    async fn load_image(&self, id: &str) -> Result<Option<Vec<u8>>>;
}
pub struct AwsImageDao {
    bucket: String,
    client: Arc<Client>,
}

impl AwsImageDao {
    pub async fn new() -> Result<AwsImageDao> {
        let bucket = "cimgbucket".to_owned();
        let region = "us-east-1";

        let region_provider = RegionProviderChain::first_try(Region::new(region))
            .or_default_provider()
            .or_else(Region::new("us-west-2"));

        println!("S3 client version: {}", PKG_VERSION);
        println!("Bucket: {}", &bucket);
        println!(
            "Region: {}",
            region_provider.region().await.unwrap().as_ref()
        );

        let shared_config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&shared_config);

        Ok(AwsImageDao {
            bucket,
            client: Arc::new(client),
        })
    }
}

#[async_trait]
impl ImageDao for AwsImageDao {
    async fn save_image(&self, id: &str, image: Vec<u8>) -> Result<()> {
        println!("saving id: {:?} image: {:?}", id, image);
        let res = upload_bytes(&self.client, &self.bucket, image, id).await?;
        // let res = upload_object(&client, &bucket, &filename, &key).await?;
        println!("upload res: {:?}", res);
        Ok(())
    }

    async fn load_image(&self, id: &str) -> Result<Option<Vec<u8>>> {
        println!("loading image: {:?}", id);
        let res = download_bytes(&self.client, &self.bucket, id).await?;
        println!("download res: {:?}", res);
        Ok(res)
    }
}

pub struct MemImageDaoImpl {
    // TODO mutex needed?
    pub state: Mutex<HashMap<String, Vec<u8>>>,
}

#[async_trait]
impl ImageDao for MemImageDaoImpl {
    async fn save_image(&self, id: &str, image: Vec<u8>) -> Result<()> {
        println!("id: {:?}", id);

        let mut s = self.state.lock().unwrap();
        println!("saving image: {:?}", image);
        s.insert(id.to_owned(), image);

        Ok(())
    }

    async fn load_image(&self, id: &str) -> Result<Option<Vec<u8>>> {
        println!("loading image: {:?}", id);

        let s = self.state.lock().unwrap();
        Ok(s.get(id).map(|o| o.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, sync::Mutex};

    use super::{ImageDao, MemImageDaoImpl};
    use crate::logger::init_logger;
    use anyhow::Result;

    #[test]
    #[ignore]
    fn test_init() -> Result<()> {
        init_logger();
        let image_dao = create_test_image_dao()?;
        Ok(())
    }

    // to be executed after test_init
    #[test]
    #[ignore]
    fn test_insert_and_load_an_image() -> Result<()> {
        init_logger();
        let _image_dao = create_test_image_dao()?;

        todo!();
    }

    fn create_test_image_dao() -> Result<Box<dyn ImageDao>> {
        Ok(Box::new(MemImageDaoImpl {
            state: Mutex::new(HashMap::new()),
        }))
    }
}

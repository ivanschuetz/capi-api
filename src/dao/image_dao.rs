use anyhow::Result;
use async_trait::async_trait;
use data_encoding::BASE64;
use sha2::Digest;
use std::{collections::HashMap, sync::Mutex};

#[async_trait]
pub trait ImageDao: Sync + Send {
    fn init(&self) -> Result<()>;

    fn save_image(&self, image: &[u8]) -> Result<()>;
    fn load_image(&self, id: &str) -> Result<Option<Vec<u8>>>;
}
pub struct ImageDaoImpl {
    // pub client: Arc<Client>,
}

#[async_trait]
impl ImageDao for ImageDaoImpl {
    fn init(&self) -> Result<()> {
        Ok(())
    }

    fn save_image(&self, image: &[u8]) -> Result<()> {
        println!("saving image: {:?}", image);
        Ok(())
    }

    fn load_image(&self, id: &str) -> Result<Option<Vec<u8>>> {
        println!("loading image: {:?}", id);
        Ok(Some(vec![]))
    }
}

pub struct MemImageDaoImpl {
    pub state: Mutex<HashMap<String, Vec<u8>>>,
}

#[async_trait]
impl ImageDao for MemImageDaoImpl {
    fn init(&self) -> Result<()> {
        Ok(())
    }

    fn save_image(&self, image: &[u8]) -> Result<()> {
        let hash = sha2::Sha512_256::digest(image);
        let encoded_hash = BASE64.encode(&hash);

        println!("encoded_hash: {:?}", encoded_hash);

        let mut s = self.state.lock().unwrap();
        s.insert(encoded_hash, image.to_vec()); // TODO maybe parameter vec

        println!("saving image: {:?}", image);
        Ok(())
    }

    fn load_image(&self, id: &str) -> Result<Option<Vec<u8>>> {
        println!("loading image: {:?}", id);

        let s = self.state.lock().unwrap();
        Ok(s.get(id).map(|o| o.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::{ImageDao, ImageDaoImpl};
    use crate::{dao::db::create_db_client, logger::init_logger};
    use anyhow::Result;
    // use tokio::test;

    #[test]
    #[ignore]
    fn test_init() -> Result<()> {
        init_logger();
        let project_dao = create_test_image_dao()?;

        project_dao.init()?;
        Ok(())
    }

    // to be executed after test_init
    #[test]
    #[ignore]
    fn test_insert_and_load_an_image() -> Result<()> {
        init_logger();
        let project_dao = create_test_image_dao()?;

        todo!();
    }

    fn create_test_image_dao() -> Result<Box<dyn ImageDao>> {
        let client = create_db_client()?;
        Ok(Box::new(ImageDaoImpl {}))
    }
}

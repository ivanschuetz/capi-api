use super::image_dao::ImageDao;
use crate::Env;
use anyhow::Result;

pub fn save_image(dao: &dyn ImageDao, env: &Env, image: &[u8]) -> Result<()> {
    dao.save_image(image)
}

pub fn load_image(dao: &dyn ImageDao, env: &Env, id: &str) -> Result<Option<Vec<u8>>> {
    dao.load_image(id)
}

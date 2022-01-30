use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::Server;

pub type Texture = Rc<three_d::Texture2D<u8>>;
pub type MaybeTexture = Rc<RefCell<Option<three_d::ThreeDResult<Texture>>>>;

#[derive(Default)]
pub struct Pool {
    pool: Rc<RefCell<BTreeMap<String, three_d::Loading<Texture>>>>,
}

impl Pool {
    pub fn preload(&self, gl: &three_d::Context, server: &impl Server, path: &str) -> MaybeTexture {
        let pool = Rc::clone(&self.pool);

        let path = server.load_asset(path);

        log::info!("Loading {:?}", path);

        let mut pool = pool.borrow_mut();

        let loading = pool.entry(path.clone()).or_insert_with(|| {
            three_d::Loading::new(gl, &[&path.clone()], move |gl, mut loaded| {
                let image = loaded.image(&path)?;
                let texture = Rc::new(three_d::Texture2D::new(&gl, &image)?);
                Ok(texture)
            })
        });
        Rc::clone(&*loading)
    }

    pub fn request(&self, gl: &three_d::Context, server: &impl Server, path: &str) -> MaybeTexture {
        let pool = self.pool.borrow();

        match pool.get(path) {
            Some(texture) => Rc::clone(&*texture),
            None => {
                drop(pool); // early drop because it is borrowed in preload()
                self.preload(gl, server, path)
            }
        }
    }
}

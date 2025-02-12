use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ByteString {
    data: Rc<[u8]>,
}

impl ByteString {
    pub fn new(data: &Rc<[u8]>) -> ByteString {
        ByteString {
            data: Rc::clone(data),
        }
    }

    pub fn get_data(&self) -> &Rc<[u8]> {
        &self.data
    }
}

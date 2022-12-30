use frame_support::inherent::Vec;
use crate::*;

pub struct App<T: Config> {
    id: T::AppId,
    name: Vec<u32>,
    owner: T::AccountId,
    created_at: T::Moment,
    star: T::Star,
}

impl <T: Config> App<T> {
       
    pub fn id(&self) -> T::AppId {
        self.id
    }    
}

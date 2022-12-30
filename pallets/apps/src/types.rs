#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, MaxEncodedLen, Decode};
use frame_support::pallet_prelude::BoundedVec;
use scale_info::TypeInfo;
use frame_support::traits::Time;
use crate::Config;

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct App<T: Config> {
    id: T::AppId,
    name: BoundedVec<u8, T::StringLimit>,
    symbol: BoundedVec<u8, T::StringLimit>,
    owner: T::AccountId,
    created_at: T::Moment,
    star: Option<T::Star>,
}

impl <T: Config> App<T> {

    pub fn create(id: T::AppId, owner: T::AccountId, name: BoundedVec<u8, T::StringLimit>, symbol: BoundedVec<u8, T::StringLimit>) -> Self {
        let now = T::AppTime::now();
        App { 
            id,
            name,
            owner,
            symbol,
            created_at: now, 
            star: None,
        }
    }
       
    pub fn id(&self) -> T::AppId {
        self.id
    }    

    pub fn name(&self) -> BoundedVec<u8, T::StringLimit> {
        self.name.clone()
    }

    pub fn symbol(&self) -> BoundedVec<u8, T::StringLimit> {
        self.symbol.clone()
    }

    pub fn owner(&self) -> T::AccountId {
        self.owner.clone()
    }

    pub fn created_at(&self) -> T::Moment {
        self.created_at
    }

    pub fn star(&self) -> Option<T::Star> {
        self.star
    }

    pub fn set_name(&mut self, name: BoundedVec<u8, T::StringLimit>) {
        self.name = name;
    }

    pub fn set_symbol(&mut self, symbol: BoundedVec<u8, T::StringLimit>) {
        self.symbol = symbol;
    }
}
use ahash::AHashMap;
use std::any::{Any, TypeId};

#[derive(Default)]
pub struct Container {
    values: AHashMap<TypeId, Box<dyn Any + Send>>,
}

impl Container {
    pub fn get_mut<T>(&mut self) -> &mut T
    where
        T: Default + Send + 'static,
    {
        self.values
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut()
            .unwrap()
    }
}

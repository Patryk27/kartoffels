mod id;
mod name;

pub use self::id::*;
pub use self::name::*;
use crate::{
    bots, clients, handle, stats, store, Bots, Client, Map, Metronome, Mode,
    Policy, RequestRx, Theme,
};
use ahash::AHashMap;
use rand::rngs::SmallRng;
use std::any::{Any, TypeId};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

pub struct World {
    pub bots: Bots,
    pub clients: Vec<Client>,
    pub events: Events,
    pub map: Map,
    pub metronome: Metronome,
    pub mode: Mode,
    pub name: Arc<WorldName>,
    pub path: Option<PathBuf>,
    pub paused: bool,
    pub policy: Policy,
    pub rng: SmallRng,
    pub rx: RequestRx,
    pub systems: Container,
    pub theme: Theme,
}

impl World {
    #[cfg(target_arch = "wasm32")]
    pub fn spawn(mut self) {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        use web_sys::WorkerGlobalScope;

        let interval = self.metronome.interval().as_millis() as i32;

        let handler = Closure::<dyn FnMut()>::new(move || {
            self.tick();
        });

        js_sys::global()
            .dyn_into::<WorkerGlobalScope>()
            .expect("couldn't find WorkerGlobalScope")
            .set_interval_with_callback_and_timeout_and_arguments(
                &handler.into_js_value().try_into().unwrap(),
                interval,
                &Default::default(),
            )
            .expect("couldn't setup event loop");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn(mut self) {
        use std::thread;
        use tokio::runtime::Handle as TokioHandle;

        let rt = TokioHandle::current();

        thread::spawn(move || {
            let _rt = rt.enter();

            loop {
                self.metronome.tick();
                self.metronome.wait();
            }
        });
    }

    pub fn tick(&mut self) {
        handle::process_requests(self);
        clients::create(self);

        if !self.paused {
            bots::spawn(self);
            bots::tick(self);
            bots::reap(self);
        }

        clients::broadcast(self);
        store::save(self);
        stats::run(self);
    }
}

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

#[derive(Default)]
pub struct Events {
    events: Container,
}

impl Events {
    pub fn send<T>(&mut self, event: T)
    where
        T: Send + 'static,
    {
        self.events.get_mut::<VecDeque<_>>().push_back(event);
    }

    pub fn recv<T>(&mut self) -> Option<T>
    where
        T: Send + 'static,
    {
        self.events.get_mut::<VecDeque<_>>().pop_front()
    }
}

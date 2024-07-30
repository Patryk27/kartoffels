mod id;
mod name;

pub use self::id::*;
pub use self::name::*;
use crate::{
    bots, clients, handle, stats, store, Bots, Client, Map, Metronome, Mode,
    Policy, RequestRx, Theme,
};
use ahash::AHashMap;
use glam::IVec2;
use rand::rngs::SmallRng;
use std::any::{Any, TypeId};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

pub struct World {
    pub bots: Bots,
    pub clients: Vec<Client>,
    pub events: Events,
    pub map: Map,
    pub mode: Mode,
    pub name: Arc<WorldName>,
    pub path: Option<PathBuf>,
    pub paused: bool,
    pub policy: Policy,
    pub rng: SmallRng,
    pub rx: RequestRx,
    pub spawn_point: Option<IVec2>,
    pub systems: Container,
    pub theme: Theme,

    #[allow(dead_code)]
    pub platform: Platform,
}

impl World {
    #[cfg(target_arch = "wasm32")]
    pub fn spawn(mut self) {
        use wasm_bindgen::closure::Closure;

        let interval = Metronome::new(cfg::SIM_HZ, cfg::SIM_TICKS)
            .interval()
            .as_millis() as i32;

        let interval_handle = self.platform.interval_handle.clone();

        let handler = Closure::<dyn FnMut()>::new(move || {
            self.tick();
        });

        *interval_handle.borrow_mut() = Some(
            web_sys::window()
                .expect("couldn't find window")
                .set_interval_with_callback_and_timeout_and_arguments(
                    &handler.into_js_value().try_into().unwrap(),
                    interval,
                    &Default::default(),
                )
                .expect("couldn't setup event loop"),
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn(mut self) {
        use std::thread;
        use tokio::runtime::Handle as TokioHandle;

        let rt = TokioHandle::current();

        thread::spawn(move || {
            let _rt = rt.enter();
            let mut metronome = Metronome::new(cfg::SIM_HZ, cfg::SIM_TICKS);

            loop {
                self.tick();

                metronome.tick();
                metronome.wait();
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

#[derive(Default)]
pub struct Platform {
    #[cfg(target_arch = "wasm32")]
    pub interval_handle: Rc<RefCell<Option<i32>>>,
}

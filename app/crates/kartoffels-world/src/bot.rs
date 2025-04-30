mod arm;
mod compass;
mod events;
mod id;
mod inventory;
mod motor;
mod radar;
mod serial;
mod timer;

pub use self::arm::*;
pub use self::compass::*;
pub use self::events::*;
pub use self::id::*;
pub use self::inventory::*;
pub use self::motor::*;
pub use self::radar::*;
pub use self::serial::*;
pub use self::timer::*;
use crate::*;
use kartoffels_cpu::TickError;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct AliveBot {
    #[serde(flatten)]
    pub body: AliveBotBody,
    pub cpu: Cpu,
}

impl AliveBot {
    pub fn new(
        rng: &mut impl RngCore,
        clock: &Clock,
        pos: IVec2,
        dir: Dir,
        mut bot: QueuedBot,
    ) -> Self {
        bot.events
            .add(clock, if bot.requeued { "reincarnated" } else { "born" });

        Self {
            cpu: Cpu::new(&bot.fw),

            body: AliveBotBody {
                arm: Default::default(),
                compass: Default::default(),
                dir,
                events: bot.events,
                fw: bot.fw,
                id: bot.id,
                inventory: Default::default(),
                motor: Default::default(),
                oneshot: bot.oneshot,
                pos,
                radar: Default::default(),
                serial: Default::default(),
                timer: BotTimer::new(rng),
            },
        }
    }

    pub fn age(&self) -> Ticks {
        Ticks::new(self.timer.ticks())
    }

    pub fn tick(&mut self, world: &mut World) -> Result<(), TickError> {
        BotTimer::tick(self);
        BotArm::tick(self);
        BotMotor::tick(self);
        BotRadar::tick(self);
        BotCompass::tick(self);

        self.cpu.tick(BotMmio {
            bot: &mut self.body,
            world,
        })?;

        Ok(())
    }
}

impl ops::Deref for AliveBot {
    type Target = AliveBotBody;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl ops::DerefMut for AliveBot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct AliveBotBody {
    pub arm: BotArm,
    pub compass: BotCompass,
    pub dir: Dir,
    pub events: BotEvents,
    pub fw: Firmware,
    pub id: BotId,
    pub inventory: BotInventory,
    pub motor: BotMotor,
    pub oneshot: bool,
    pub pos: IVec2,
    pub radar: BotRadar,
    pub serial: BotSerial,
    pub timer: BotTimer,
}

impl AliveBotBody {
    pub(crate) const FELL_INTO_VOID: IVec2 = ivec2(i32::MAX, i32::MAX);

    fn load(&mut self, addr: u32) -> Result<u32, ()> {
        BotTimer::load(self, addr)
            .or_else(|_| BotMotor::load(self, addr))
            .or_else(|_| BotArm::load(self, addr))
            .or_else(|_| BotRadar::load(self, addr))
            .or_else(|_| BotCompass::load(self, addr))
    }

    fn store(
        &mut self,
        world: &mut World,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        BotTimer::store(self, addr, val)
            .or_else(|_| BotSerial::store(self, addr, val))
            .or_else(|_| BotMotor::store(self, world, addr, val))
            .or_else(|_| BotArm::store(self, world, addr, val))
            .or_else(|_| BotRadar::store(self, world, addr, val))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadBot {
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub id: BotId,
    pub serial: Arc<VecDeque<u32>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedBot {
    pub dir: Option<Dir>,
    pub events: BotEvents,
    pub fw: Firmware,
    pub id: BotId,
    pub oneshot: bool,
    pub pos: Option<IVec2>,
    pub requeued: bool, // TODO rename to `reincarnated`
    pub serial: BotSerial,
}

struct BotMmio<'a> {
    bot: &'a mut AliveBotBody,
    world: &'a mut World,
}

impl Mmio for BotMmio<'_> {
    fn load(&mut self, addr: u32) -> Result<u32, ()> {
        self.bot.load(addr)
    }

    fn store(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        self.bot.store(self.world, addr, val)
    }
}

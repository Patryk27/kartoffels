mod header;
mod migrations;

use self::header::*;
use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedWorld<'a> {
    pub bots: MaybeOwned<'a, Bots>,
    pub lives: MaybeOwned<'a, Lives>,
    pub map: MaybeOwned<'a, Map>,
    pub name: String,
    pub policy: MaybeOwned<'a, Policy>,
    pub rng: MaybeOwned<'a, ChaCha8Rng>,
    pub theme: Option<MaybeOwned<'a, Theme>>,
}

pub fn save(world: &World) -> WorldBuffer {
    let world = SerializedWorld {
        bots: MaybeOwned::Borrowed(&world.bots),
        lives: MaybeOwned::Borrowed(&world.lives),
        map: MaybeOwned::Borrowed(&world.map),
        name: world.name.load().to_string(),
        policy: MaybeOwned::Borrowed(&world.policy),
        rng: MaybeOwned::Borrowed(&world.rng),
        theme: world.theme.as_ref().map(MaybeOwned::Borrowed),
    };

    let mut buf = Vec::new();

    Header::default().write(&mut buf);
    ciborium::into_writer(&world, &mut buf).unwrap();

    WorldBuffer::new(buf)
}

pub fn load(buf: WorldBuffer) -> Result<SerializedWorld<'static>> {
    let mut buf = Cursor::new(buf.into_vec());

    let header = Header::read(&mut buf)
        .context("couldn't read header")?
        .validated()
        .context("couldn't validate header")?;

    let this =
        ciborium::from_reader(&mut buf).context("couldn't read state")?;

    let this = migrations::run(header.version(), migrations::version(), this)
        .context("couldn't migrate state")?;

    let this = this.deserialized().context("couldn't deserialize state")?;

    Ok(this)
}

#[derive(Clone, Debug)]
pub struct WorldBuffer(Vec<u8>);

impl WorldBuffer {
    pub fn new(buf: Vec<u8>) -> Self {
        Self(buf)
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

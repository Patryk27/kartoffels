#[derive(Clone, Copy, Debug)]
pub struct Prefab {
    pub id: &'static str,
    pub name: &'static str,
    pub source: &'static [u8],
}

macro_rules! prefabs {
    (
        $(
            $id:ident: {
                name: $name:literal,
            },
        )*
    ) => {
        $(
            pub static $id: &[u8] = include_bytes!(
                env!(concat!("KARTOFFELS_BOT_", stringify!($id)))
            );
        )*

        pub static ALL: &[Prefab] = &[
            $(
                Prefab {
                    id: stringify!($id),
                    name: $name,
                    source: $id,
                },
            )*
        ];
    }
}

prefabs! {
    CHL_ACYCLIC_MAZE: {
        name: "chl-acyclic-maze",
    },

    CHL_FLIGHT_SYNDROME_ENEMY: {
        name: "chl-flight-syndrome-enemy",
    },

    DUMMY: {
        name: "dummy",
    },

    ROBERTO: {
        name: "roberto",
    },
}

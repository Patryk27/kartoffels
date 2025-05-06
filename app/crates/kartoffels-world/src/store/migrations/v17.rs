use ciborium::Value;
use kartoffels_utils::{CborMapExt, CborValueExt};

pub fn run(world: &mut Value) {
    for bot in world.query_mut("/bots/alive/*") {
        let bot = bot.as_map_mut().unwrap();

        bot.rename_entry("timer", "clock");

        bot.add_entry(
            "irq",
            Vec::new()
                .with_entry("memory", vec![0; 256])
                .with_entry("args", vec![0; 32])
                .with_entry("pending", 0)
                .into_map(),
        );
    }

    for clock in world.query_mut("/bots/alive/*/clock") {
        let clock = clock.as_map_mut().unwrap();

        for timer in ["timer0", "timer1", "timer2"] {
            clock.add_entry(
                timer,
                Vec::new()
                    .with_entry("cfg", 0)
                    .with_entry("acc", 0)
                    .with_entry("max", 0)
                    .with_entry("ticks", 0)
                    .into_map(),
            );
        }
    }

    for compass in world.query_mut("/bots/alive/*/compass") {
        compass
            .as_map_mut()
            .unwrap()
            .rename_entry("next_measurement_in", "cooldown");
    }

    for cpu in world.query_mut("/bots/alive/*/cpu") {
        cpu.as_map_mut()
            .unwrap()
            .add_entry("prev_pc", 0)
            .add_entry("prev_regs", [0; 32]);
    }
}

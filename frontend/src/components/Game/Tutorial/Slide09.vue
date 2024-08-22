<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.alterUi((ui) => {
  ui.enableUploadBot = true;
  ui.highlightUploadBot = true;
});

ctrl.resume();
ctrl.getLocalServer().destroyAllBots();

ctrl.onOnce("server.bot-create", () => {
  ctrl.openTutorialSlide(10);
});
</script>

<template>
  <main>
    <p>
      radar is the second peripheral - it allows to scan the bot's neighbourhood
      through these four functions:
    </p>

    <ul class="compact">
      <li><kbd>radar_scan_3x3()</kbd></li>
      <li><kbd>radar_scan_5x5()</kbd></li>
      <li><kbd>radar_scan_7x7()</kbd></li>
      <li><kbd>radar_scan_9x9()</kbd></li>
    </ul>

    <p>
      the scan is always square-shaped (3x3, 5x5 etc.) and already rotated
      according to the robot's direction
    </p>

    <p>
      e.g. when using the 3x3 scan, the <kbd>scan[0][1]</kbd> cell always
      describes what's directly in front of the robot and corresponds to the
      place the robot will go when you call <kbd>motor_step()</kbd>
    </p>

    <p>
      to put this into practice, let's make use of the radar to prevent our bot
      from falling:
    </p>

    <pre>
#[no_mangle]
fn main() {
    loop {
        motor_step();

        let scan = radar_scan_3x3();

        // `scan` is a 2D char-array (`[[char; 3]; 3]`)
        // representing our robot's neighbourhood, with
        // the robot always at the center
        //
        // basically, if the map looks like:
        //
        // ```
        // A B C
        // D @ F
        // G H I
        // ```
        //
        // ... then `scan` will say:
        //
        // ```
        // [
        //   ['A', 'B', 'C'],
        //   ['D', '@', 'F'],
        //   ['G', 'H', 'I']
        // ]
        // ```
        //
        // in practice, for a 3x3 scan:
        //
        // - scan[1][1] is `@` (i.e. us),
        // - scan[0][1] is the tile in front of us
        // - scan[2][1] is the tile behind us
        // - scan[1][0] is the tile to our left
        // - scan[1][2] is the tile to our right

        // if going forward would cause us to fall, turn
        // right
        if scan[0][1] == ' ' {
            motor_turn_right();
        }
    }
}</pre
    >

    <p>
      now, update your <kbd>main.rs</kbd>, build the project and upload bot v2.0
    </p>
  </main>
</template>

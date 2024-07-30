<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.alterUi((ui) => {
  ui.enableUploadBot = true;
  ui.highlightUploadBot = true;
});
</script>

<template>
  <p>
    radar is the second peripheral - it allows to scan the bot's neighbourhood
    through these four functions:
  </p>

  <ul>
    <li><kbd>radar_scan_3()</kbd></li>
    <li><kbd>radar_scan_5()</kbd></li>
    <li><kbd>radar_scan_7()</kbd></li>
    <li><kbd>radar_scan_9()</kbd></li>
  </ul>

  <p>
    the scan is always a <kbd>D</kbd> x <kbd>D</kbd> square, and the number at
    the end of the function tells you how large the scan will be
  </p>

  <p>e.g. <kbd>radar_scan_3()</kbd> performs a 3x3 scan</p>

  <p>
    to put this into practice, let's make use of the radar to prevent our bot
    from falling:
  </p>

  <pre>
#[no_mangle]
fn main() {
    loop {
        motor_step();

        let scan = radar_scan_3();

        // `scan` is a 2D char-array (`[[char; 3]; 3]`)
        // representing our robot's neighbourhood
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

        // if there's no floor in front of us, turn right
        if scan[0][1] == ' ' {
            motor_turn(1);
        }
    }
}
  </pre>

  <p>
    now, update your <kbd>main.rs</kbd>, build the project and submit bot v2.0
  </p>
</template>

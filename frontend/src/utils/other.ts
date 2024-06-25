export function durationToHuman(sec) {
  if (sec < 60) {
    return `${sec}s`;
  } else {
    const mins = Math.floor(sec / 60);
    const secs = sec % 60;

    return `${mins}m ${secs}s`;
  }
}

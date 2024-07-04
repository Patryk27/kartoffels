export function durationToHuman(sec) {
  if (sec < 60) {
    return `${sec}s`;
  } else {
    const mins = Math.floor(sec / 60);
    const secs = sec % 60;

    return `${mins}m ${secs}s`;
  }
}

export function ordinal(nth) {
  switch (nth % 100) {
    case 11:
    case 12:
    case 13:
      return "th";
  }

  switch (nth % 10) {
    case 3:
      return "rd";

    case 2:
      return "nd";

    case 1:
      return "st";

    default:
      return "th";
  }
}

export function isValidBotId(id: string): boolean {
  return id.match(/^([0-9a-fA-F]{4}-){3}[0-9a-fA-F]{4}$/) != null;
}

export function botIdToColor(id: string, mode: string = "fg"): string {
  let hue = stringToHash(id) % 360;

  if (mode == "fg") {
    return `hsl(${hue}, 100%, 50%)`;
  } else {
    return `hsl(${hue}, 100%, 33%)`;
  }
}

function stringToHash(str: string): number {
  let hash = 0;

  for (let i = 0; i < str.length; i++) {
    hash = hash * 31 + (str.charCodeAt(i) | 0);
  }

  return hash;
}

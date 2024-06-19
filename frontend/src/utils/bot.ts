export function botIdToColor(id: string): string {
  const hue = stringToHash(id) % 360;

  return `hsl(${hue}, 100%, 50%)`;
}

function stringToHash(str: string): int {
  let hash = 0;

  for (let i = 0; i < str.length; i++) {
    hash = hash * 31 + (str.charCodeAt(i) | 0);
  }

  return hash;
}

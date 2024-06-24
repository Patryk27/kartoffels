// TODO store camera position as well
interface Session {
  worldId: int;
  botId?: int;
}

export function loadSession(): Session | null {
  const session = localStorage.getItem('session');

  if (session == null) {
    return null;
  } else {
    return JSON.parse(session);
  }
}

export function storeSession(session: Session): void {
  localStorage.setItem('session', JSON.stringify(session));
}

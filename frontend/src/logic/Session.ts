// TODO store camera position as well
export interface Session {
  worldId: string;
  botId?: string;
}

export function loadSession(): Session | null {
  const session = localStorage.getItem("session");

  if (session) {
    return JSON.parse(session);
  } else {
    return null;
  }
}

export function storeSession(session: Session): void {
  localStorage.setItem("session", JSON.stringify(session));
}
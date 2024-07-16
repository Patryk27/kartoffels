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

// Keeps track which bots were uploaded by the player, so that we can highlight
// them differently etc.
export class PlayerBots {
  worldId: string;
  bots: Map<String, Date>;

  constructor(worldId: string) {
    const bots =
      worldId == "sandbox" ? null : localStorage.getItem(`${worldId}.bots`);

    if (bots) {
      this.bots = new Map(JSON.parse(bots));
    } else {
      this.bots = new Map();
    }

    this.worldId = worldId;
  }

  add(botId: string): void {
    this.bots.set(botId, new Date());
    this.gc();
    this.save();
  }

  has(botId: string): boolean {
    return this.bots.has(botId);
  }

  // Removes old bots, so that we don't keep a potentially huge map in the
  // memory.
  private gc(): void {
    while (this.bots.size >= 4096) {
      const bots = [];

      for (const [id, uploadedAt] of this.bots) {
        bots.push({ id, uploadedAt });
      }

      bots.sort((a, b) => {
        return b.uploadedAt.getTime() - a.uploadedAt.getTime();
      });

      const oldestBotId = bots[0].id;

      this.bots.delete(oldestBotId);
    }
  }

  private save(): void {
    if (this.worldId == "sandbox") {
      return;
    }

    localStorage.setItem(
      `${this.worldId}.bots`,
      JSON.stringify(Array.from(this.bots)),
    );
  }
}

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
  private worldId?: string;
  private bots: Map<String, Date>;

  constructor(worldId: string) {
    switch (worldId) {
      case "sandbox":
      case "tutorial":
        this.worldId = null;
        break;

      default:
        this.worldId = worldId;
        break;
    }

    this.bots = new Map();

    if (this.worldId) {
      const bots = localStorage.getItem(`${worldId}.bots`);

      if (bots) {
        for (const [id, uploadedAt] of JSON.parse(bots)) {
          this.bots.set(id, new Date(uploadedAt));
        }
      }
    }
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
    while (this.bots.size > 4096) {
      const bots = [];

      for (const [id, uploadedAt] of this.bots) {
        bots.push({ id, uploadedAt });
      }

      bots.sort((a, b) => {
        return a.uploadedAt.getTime() - b.uploadedAt.getTime();
      });

      const oldestBotId = bots[0].id;

      this.bots.delete(oldestBotId);
    }
  }

  private save(): void {
    if (this.worldId) {
      localStorage.setItem(
        `${this.worldId}.bots`,
        JSON.stringify(Array.from(this.bots)),
      );
    }
  }
}

import { LocalServer, type Server } from "@/logic/Server";
import { ref, type Ref } from "vue";

export class GameCtrl {
  server: Server;
  paused: Ref<boolean>;

  ui: Ref<GameUi>;
  events: Map<String, Array<EventHandler>>;
  tutorialSlide: Ref<number>;

  constructor(server: Server, paused: Ref<boolean>) {
    // Comes handy for debugging:
    (<any>window).ctrl = this;

    this.server = server;
    this.paused = paused;

    this.ui = ref({
      enableConnectToBot: true,
      enableDisconnectFromBot: true,
      enableHelp: true,
      enablePause: true,
      enableUploadBot: true,
      highlightPause: false,
      highlightUploadBot: false,
      showBotList: true,
    });

    this.events = new Map();
    this.tutorialSlide = ref(null);
  }

  on(event: string, fn: (payload: any) => void, once: boolean = false): void {
    log("on()", event, fn, once);

    if (!this.events.has(event)) {
      this.events.set(event, []);
    }

    this.events.get(event).push({ fn, once });
  }

  onOnce(event: string, handler: (payload: any) => void): void {
    this.on(event, handler, true);
  }

  waitFor(event: string): Promise<any> {
    return new Promise((resolve) => {
      this.onOnce(event, resolve);
    });
  }

  emit(event: string, payload: any = null): void {
    log("emit()", event, payload);

    if (!this.events.has(event)) {
      return;
    }

    let handlers = this.events.get(event);

    for (const handler of handlers) {
      handler.fn(payload);
    }

    handlers = handlers.filter((handler) => {
      return !handler.once;
    });

    this.events.set(event, handlers);
  }

  openTutorialSlide(slide: number): void {
    this.tutorialSlide.value = slide;
    this.emit("tutorial.before-slide");
  }

  alterUi(f: (ui: GameUi) => void): void {
    f(this.ui.value);
  }

  pause(): void {
    this.paused.value = true;
  }

  resume(): void {
    this.paused.value = false;
  }

  getLocalServer(): LocalServer {
    if (this.server instanceof LocalServer) {
      return this.server;
    } else {
      throw "called getLocalServer() on a non-local server";
    }
  }

  /// Returns a stream that yields ids of killed bots.
  async listenForKilledBots(): Promise<ReadableStream<string>> {
    const events = await this.getLocalServer().listen();

    return new ReadableStream({
      async start(ctrl) {
        for await (const event of events) {
          if (event.ty == "bot-killed") {
            ctrl.enqueue(event.id);
          }
        }
      },
    });
  }

  /// Returns a promise that resolves when any bot is killed.
  async onceAnyBotIsKilled(): Promise<void> {
    const killedBots = await this.listenForKilledBots();

    await killedBots.getReader().read();
  }

  /// Returns a promise that resolves once given bot is killed.
  onceBotIsKilled(id: string): Promise<void> {
    return this.onceBotsAreKilled([id]);
  }

  /// Returns a promise that resolves once all given bots are killed.
  onceBotsAreKilled(ids: string[]): Promise<void> {
    return new Promise(async (resolve, reject) => {
      for await (const id2 of await this.listenForKilledBots()) {
        ids = ids.filter((id) => id != id2);

        if (ids.length == 0) {
          resolve();
          return;
        }
      }

      reject();
    });
  }

  /// Creates a task that decrements given counter and returns a promise that
  /// resolves once this counter gets down to zero.
  onceTimerIsCompleted(timer: Ref<number>): Promise<void> {
    return new Promise((resolve) => {
      const handle = setInterval(() => {
        timer.value -= 1;

        if (timer.value <= 0) {
          resolve(null);
          clearInterval(handle);
        }
      }, 1000);
    });
  }
}

export interface GameUi {
  enableConnectToBot: boolean;
  enableDisconnectFromBot: boolean;
  enableHelp: boolean;
  enablePause: boolean;
  enableUploadBot: boolean;
  highlightPause: boolean;
  highlightUploadBot: boolean;
  showBotList: boolean;
}

interface EventHandler {
  fn: (payload: any) => void;
  once: boolean;
}

function log(...data: any[]) {
  console.log("[ctrl]", ...data);
}

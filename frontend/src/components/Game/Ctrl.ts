import { LocalServer, type Server } from "@/logic/Server";
import { ref, type Ref } from "vue";

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

export class GameCtrl {
  server: Server;
  paused: Ref<boolean>;

  ui: Ref<GameUi>;
  events: Map<String, Array<EventHandler>>;
  tutorialSlide: Ref<number>;

  constructor(server: Server, paused: Ref<boolean>) {
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

  on(event: string, fn: () => void, once: boolean = false): void {
    log("on()", event, fn, once);

    if (!this.events.has(event)) {
      this.events.set(event, []);
    }

    this.events.get(event).push({ fn, once });
  }

  onOnce(event: string, handler: () => void): void {
    this.on(event, handler, true);
  }

  emit(event: string): void {
    log("emit()", event);

    if (!this.events.has(event)) {
      return;
    }

    let handlers = this.events.get(event);

    for (const handler of handlers) {
      handler.fn();
    }

    handlers = handlers.filter((handler) => {
      return !handler.once;
    });

    this.events.set(event, handlers);
  }

  openTutorialSlide(slide: number): void {
    this.tutorialSlide.value = slide;
    this.emit("tutorial.before-slide");

    // HACK Slides attach event hooks only after they are mounted, but since
    // we've modified `tutorialSlide` just now, the slide won't get mounted up
    // until the next frame, so... let's wait for the next frame.
    setTimeout(() => {
      this.emit(`tutorial.slide.${slide}`);
    }, 0);
  }

  hideTutorial(): void {
    this.tutorialSlide.value = null;
  }

  onTutorialSlide(id: number, handler: () => void): void {
    this.onOnce(`tutorial.slide.${id}`, handler);
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
}

interface EventHandler {
  fn: () => void;
  once: boolean;
}

function log(...data: any[]) {
  console.log("[ctrl]", ...data);
}

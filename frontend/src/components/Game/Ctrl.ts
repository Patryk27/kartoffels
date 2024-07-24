import { ref, type Ref } from "vue";

export interface GameUi {
  btnHelpDisabled: boolean;
  btnPauseDisabled: boolean;
  btnConnectToBotDisabled: boolean;
  btnUploadBotDisabled: boolean;
  btnUploadBotHighlighted: boolean;
}

export class GameCtrl {
  ui: Ref<GameUi>;
  events: Map<String, () => void>;
  postponedEmits: Set<String>;
  paused: Ref<boolean>;
  tutorialSlide: Ref<number>;

  constructor(paused: Ref<boolean>) {
    this.ui = ref({
      btnHelpDisabled: false,
      btnPauseDisabled: false,
      btnConnectToBotDisabled: false,
      btnUploadBotDisabled: false,
      btnUploadBotHighlighted: false,
    });

    this.events = new Map();
    this.postponedEmits = new Set();
    this.paused = paused;
    this.tutorialSlide = ref(null);
  }

  on(event: string, handler: () => void): void {
    this.events.set(event, handler);

    if (this.postponedEmits.delete(event)) {
      handler();
    }
  }

  onSlide(id: number, handler: () => void): void {
    this.on(`tutorial.slide.${id}`, handler);
  }

  emit(event: string, canPostpone: boolean = false): void {
    if (this.events.has(event)) {
      this.events.get(event)();
    } else {
      // Postponing is a hacky approach to solve a tiiiny race condition between
      // slide transitions.
      //
      // When we call `openSlide()`, we do two things: change the currently
      // active slide and emit an event, so that the next slide can prepare its
      // environment.
      //
      // But slides only get setup once the `tutorialSlide` is changed, so
      // without this postponing mechanism, we wouldn't be able to get beyond
      // the first slide (i.e. changing the slide number only activates the
      // slide, and thus makes it register the event, on the next frame).
      if (canPostpone) {
        this.postponedEmits.add(event);
      }
    }
  }

  waitFor(event: string): Promise<void> {
    return new Promise((resolve, _) => {
      this.on(event, resolve);
    });
  }

  openSlide(id: number): void {
    this.tutorialSlide.value = id;
    this.emit(`tutorial.slide.${id}`, true);
  }

  disableButton(id: string): void {
    this.setButtonDisabled(id, true);
  }

  enableButton(id: string): void {
    this.setButtonDisabled(id, false);
  }

  highlightButton(id: string): void {
    this.setButtonHighlighted(id, true);
  }

  unhighlightButton(id: string): void {
    this.setButtonHighlighted(id, false);
  }

  setButtonDisabled(id: string, disabled: boolean): void {
    switch (id) {
      case "help":
        this.ui.value.btnHelpDisabled = disabled;
        break;

      case "pause":
        this.ui.value.btnPauseDisabled = disabled;
        break;

      case "connectToBot":
        this.ui.value.btnConnectToBotDisabled = disabled;
        break;

      case "uploadBot":
        this.ui.value.btnUploadBotDisabled = disabled;
        break;
    }
  }

  setButtonHighlighted(id: string, highlighted: boolean): void {
    switch (id) {
      case "uploadBot":
        this.ui.value.btnUploadBotHighlighted = highlighted;
        break;
    }
  }

  pause(): void {
    this.paused.value = true;
  }

  resume(): void {
    this.paused.value = false;
  }
}

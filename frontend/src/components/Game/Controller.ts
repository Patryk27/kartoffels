import { ref, type Ref } from "vue";

export interface GameUi {
  btnHelpDisabled: boolean;
  btnPauseDisabled: boolean;
  btnConnectToBotDisabled: boolean;
  btnUploadBotDisabled: boolean;
  btnUploadBotHighlighted: boolean;
}

export class GameController {
  ui: Ref<GameUi>;
  events: Map<String, () => void>;
  helpId: Ref<number>;

  constructor() {
    this.ui = ref({
      btnHelpDisabled: false,
      btnPauseDisabled: false,
      btnConnectToBotDisabled: false,
      btnUploadBotDisabled: false,
      btnUploadBotHighlighted: false,
    });

    this.events = new Map();
    this.helpId = ref(null);
  }

  on(event: string, handler: () => void) {
    this.events.set(event, handler);
  }

  emit(event: string): void {
    if (this.events.has(event)) {
      this.events.get(event)();
    }
  }

  waitFor(event: string): Promise<void> {
    return new Promise((resolve, _) => {
      this.on(event, resolve);
    });
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

  openHelp(id: number): void {
    this.helpId.value = id;
  }
}

import "@xterm/xterm/css/xterm.css";
import pako from "pako";
import { ClipboardAddon } from "@xterm/addon-clipboard";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { WebglAddon } from "@xterm/addon-webgl";

const $app = document.getElementById("app");

document.fonts.ready.then(() => {
  let isConnected = false;

  const term = new Terminal({
    rows: 0,
    cols: 0,
    fontSize: 24,
    fontFamily: "Fira Code",
  });

  const termFit = new FitAddon();

  term.open($app);
  term.write("connecting...");
  term.loadAddon(termFit);
  term.loadAddon(new WebglAddon());
  term.loadAddon(new ClipboardAddon());

  term.attachCustomKeyEventHandler((event) => {
    const isCtrl = (key) => {
      return event.type === "keydown" && event.ctrlKey && event.key === key;
    };

    // Don't catch C-c and C-v, so that they invoke the good-old copy and paste
    if (isCtrl("c") || isCtrl("v")) {
      return false;
    }

    // Don't catch C-r, unless we're disconnected - then allow for C-r, so that
    // the user can refresh the page
    if (isCtrl("r") && !isConnected) {
      return false;
    }
  });

  const resizeObserver = new ResizeObserver(() => {
    termFit.fit();
  });

  resizeObserver.observe($app);

  // ---

  const socket = new WebSocket(`${import.meta.env.VITE_API_URL}`);

  socket.binaryType = "arraybuffer";

  socket.onopen = () => {
    isConnected = true;

    // After connecting, send a "hello!" message so that the backend knows the
    // size of our terminal
    socket.send(
      JSON.stringify({
        cols: term.cols,
        rows: term.rows,
      }),
    );

    term.loadAddon(
      new TermSocketAddon(socket, () => {
        handleBotCreate();
      }),
    );

    term.onResize((event) => {
      const packet = new Uint8Array(3);

      packet[0] = 0x04;
      packet[1] = Math.min(event.cols, 255);
      packet[2] = Math.min(event.rows, 255);

      socket.send(packet);
    });

    term.focus();
  };

  socket.onerror = () => {
    term.reset();
    term.write(
      "ouch, the server is unreachable - try again in a moment\r\n\r\n",
    );
    term.write("");
    term.write("if this issue persists, lemme know:\r\n");
    term.write("> https://github.com/Patryk27/kartoffels");

    isConnected = false;
  };

  socket.onclose = () => {
    if (isConnected) {
      term.clear();
      term.write(
        "ouch, lost connection to the server - try again in a moment\r\n\r\n",
      );
      term.write("");
      term.write("if this issue persists, lemme know:\r\n");
      term.write("> https://github.com/Patryk27/kartoffels");
    }

    isConnected = false;
  };

  window.onbeforeunload = () => {
    socket.onclose = null;
  };

  // ---

  function handleBotCreate() {
    const input = document.createElement("input");

    input.type = "file";

    input.oncancel = () => {
      // HACK backend knows that an empty paste means that the user has aborted
      //      the uploading; ideally we'd have a dedicated message for that, but
      //      well
      term.paste("");
    };

    input.onchange = (event) => {
      const reader = new FileReader();

      reader.onload = () => {
        term.paste(
          btoa(String.fromCodePoint(...new Uint8Array(reader.result))),
        );
      };

      reader.readAsArrayBuffer(event.target.files[0]);
    };

    input.click();
  }
});

// Proxy between WebSocket and xterm.js.
//
// We can't use the built-in `AttachAddon`, since backend sends us gzipped
// messages that that addon can't handle.
class TermSocketAddon {
  constructor(socket, onBotCreateRequested) {
    this.socket = socket;
    this.onBotCreateRequested = onBotCreateRequested;
  }

  activate(term) {
    this.socket.addEventListener("message", (event) => {
      const data = pako.ungzip(event.data);

      if (data.length == 1 && data[0] == 0x04) {
        this.onBotCreateRequested();
      } else {
        term.write(data);
      }
    });

    term.onData((data) => {
      this.socket.send(data);
    });
  }
}

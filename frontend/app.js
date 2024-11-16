import "@xterm/xterm/css/xterm.css";
import { AttachAddon } from "@xterm/addon-attach";
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

  socket.onopen = () => {
    isConnected = true;

    socket.send(
      JSON.stringify({
        cols: term.cols,
        rows: term.rows,
      }),
    );

    term.loadAddon(
      new AttachAddon(socket, {
        bidirectional: true,
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

  socket.onmessage = (event) => {
    const msg = new Uint8Array(event.data);

    if (msg.length == 1 && msg[0] == 0x04) {
      handleBotCreate();
    }
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

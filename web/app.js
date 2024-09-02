import { Terminal } from "@xterm/xterm";
import { AttachAddon } from "@xterm/addon-attach";
import { FitAddon } from "@xterm/addon-fit";
import { WebglAddon } from "@xterm/addon-webgl";
import "@xterm/xterm/css/xterm.css";

const $app = document.getElementById("app");

document.fonts.ready.then(() => {
  const term = new Terminal({
    rows: 0,
    cols: 0,
    fontSize: 28,
    fontFamily: "Fira Code",
  });

  const termFit = new FitAddon();

  term.open($app);
  term.write("connecting...");
  term.loadAddon(termFit);
  term.loadAddon(new WebglAddon());

  const socket = new WebSocket("ws:localhost:1313");

  socket.onopen = () => {
    term.loadAddon(
      new AttachAddon(socket, {
        bidirectional: true,
      }),
    );

    term.onResize((event) => {
      const packet = new Uint8Array(3);

      packet[0] = 0x04;
      packet[1] = event.cols;
      packet[2] = event.rows;

      socket.send(packet);
    });

    const resizeObserver = new ResizeObserver(() => {
      termFit.fit();
    });

    resizeObserver.observe($app);

    setTimeout(() => {
      term.focus();
      termFit.fit();
    }, 100);
  };

  socket.onmessage = (event) => {
    const msg = new Uint8Array(event.data);

    if (msg.length == 1 && msg[0] == 0x04) {
      handleBotCreate();
    }
  };

  socket.onclose = () => {
    setTimeout(() => {
      term.reset();

      term.write(
        "error: lost connection to the server - try refreshing the page",
      );
    }, 150);
  };

  window.onbeforeunload = () => {
    socket.onclose = null;
  };

  function handleBotCreate() {
    const input = document.createElement("input");

    input.type = "file";

    input.onchange = (event) => {
      const reader = new FileReader();

      reader.onload = () => {
        term.paste(btoa(new Uint8Array(reader.result)));
      };

      reader.readAsArrayBuffer(event.target.files[0]);
    };

    input.click();
  }
});
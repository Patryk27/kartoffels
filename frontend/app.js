import "@xterm/xterm/css/xterm.css";
import { AttachAddon } from "@xterm/addon-attach";
import { ClipboardAddon } from "@xterm/addon-clipboard";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { WebglAddon } from "@xterm/addon-webgl";

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
  term.loadAddon(new ClipboardAddon());

  term.attachCustomKeyEventHandler((event) => {
    // Prevent xterm from catching C-v, so that it invokes the usual paste
    // event - better for the UX
    if (event.type === "keydown" && event.key === "v" && event.ctrlKey) {
      return false;
    }
  });

  const socket = new WebSocket(`${import.meta.env.VITE_API_URL}`);

  socket.onopen = () => {
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
      term.write("\n\rconnection to server closed");
    }, 150);
  };

  window.onbeforeunload = () => {
    socket.onclose = null;
  };

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

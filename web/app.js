import { Terminal } from "@xterm/xterm";
import { AttachAddon } from "@xterm/addon-attach";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

const $app = document.getElementById("app");

document.fonts.ready.then(() => {
  const term = new Terminal({
    rows: 0,
    cols: 0,
    fontSize: 26,
    fontFamily: "Inconsolata",
  });

  const termFit = new FitAddon();

  term.open($app);
  term.write("connecting...");
  term.loadAddon(termFit);

  const socket = new WebSocket("ws://localhost:1313");

  term.loadAddon(
    new AttachAddon(socket, {
      bidirectional: true,
    }),
  );

  socket.onopen = () => {
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
});

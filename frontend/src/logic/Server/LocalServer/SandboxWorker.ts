import init, { Sandbox } from "kartoffels-sandbox";
import wasmUrl from "kartoffels-sandbox/kartoffels_sandbox_bg.wasm?url";

let sandbox: Promise<any> | Sandbox | undefined = undefined;
let listenerIdx = 0;

onmessage = async (event) => {
  const data = event.data;

  switch (data.op) {
    case "init":
      sandbox = init(wasmUrl).then(() => {
        sandbox = new Sandbox(data.config);
      });

      break;

    case "uploadBot":
      try {
        if (sandbox instanceof Promise) {
          await sandbox;
        }

        const id = await (sandbox as Sandbox).upload_bot(data.src);

        postMessage({
          op: "uploadBot.response",
          response: {
            status: "ok",
            result: {
              id,
            },
          },
        });
      } catch (error) {
        postMessage({
          op: "uploadBot.response",
          response: {
            status: "err",
            error,
          },
        });
      }

      break;

    case "destroyBot":
      if (sandbox instanceof Promise) {
        await sandbox;
      }

      (sandbox as Sandbox).destroy_bot(data.id);

      break;

    case "restartBot":
      if (sandbox instanceof Promise) {
        await sandbox;
      }

      (sandbox as Sandbox).restart_bot(data.id);

      break;

    case "join":
      try {
        if (sandbox instanceof Promise) {
          await sandbox;
        }

        listenerIdx += 1;

        const thisListenerIdx = listenerIdx;
        const events = await (sandbox as Sandbox).join(data.botId);

        postMessage({
          op: "join.response",
          response: {
            status: "ok",
            result: {
              listenerIdx,
            },
          },
        });

        for await (const event of events) {
          if (listenerIdx > thisListenerIdx) {
            break;
          }

          postMessage({
            op: "join.update",
            event,
            listenerIdx,
          });
        }
      } catch (error) {
        postMessage({
          op: "join.response",
          response: {
            status: "err",
          },
        });
      }

      break;

    case "leave":
      listenerIdx += 1;
      break;
  }
};

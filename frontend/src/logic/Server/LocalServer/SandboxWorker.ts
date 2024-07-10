import init, { Sandbox } from "kartoffels-sandbox";
import wasmUrl from "kartoffels-sandbox/kartoffels_sandbox_bg.wasm?url";

let sandbox: Promise<any> | Sandbox | undefined = undefined;
let listenerIdx = 0;

function handleInit(config: any): void {
  sandbox = init(wasmUrl).then(() => {
    sandbox = new Sandbox(config);
  });
}

async function getSandbox(): Promise<Sandbox> {
  // If the sandbox is still being initialized, wait for it
  if (sandbox instanceof Promise) {
    await sandbox;
  }

  return sandbox;
}

async function handleJoin(id?: string): Promise<void> {
  try {
    const sandbox = await getSandbox();

    listenerIdx += 1;

    const thisListenerIdx = listenerIdx;
    const events = await sandbox.join(id);

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
}

async function handlePause(paused: boolean): Promise<void> {
  await (await getSandbox()).pause(paused);
}

function handleLeave(): void {
  listenerIdx += 1;
}

async function handleUploadBot(src: Uint8Array): Promise<void> {
  try {
    const sandbox = await getSandbox();
    const id = await sandbox.upload_bot(src);

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
}

async function handleSpawnPrefabBot(ty: string): Promise<void> {
  try {
    const sandbox = await getSandbox();
    const id = await sandbox.spawn_prefab_bot(ty);

    postMessage({
      op: "spawnPrefabBot.response",
      response: {
        status: "ok",
        result: {
          id,
        },
      },
    });
  } catch (error) {
    postMessage({
      op: "spawnPrefabBot.response",
      response: {
        status: "err",
        error,
      },
    });
  }
}

async function handleDestroyBot(id: string): Promise<void> {
  (await getSandbox()).destroy_bot(id);
}

async function handleRestartBot(id: string): Promise<void> {
  (await getSandbox()).restart_bot(id);
}

// ---

onmessage = (event) => {
  const data = event.data;

  console.log("[sandbox-worker] processing message:", data);

  switch (data.op) {
    case "init":
      handleInit(data.config);
      break;

    case "join":
      handleJoin(data.botId);
      break;

    case "pause":
      handlePause(data.paused);
      break;

    case "leave":
      handleLeave();
      break;

    case "uploadBot":
      handleUploadBot(data.src);
      break;

    case "spawnPrefabBot":
      handleSpawnPrefabBot(data.ty);
      break;

    case "destroyBot":
      handleDestroyBot(data.id);
      break;

    case "restartBot":
      handleRestartBot(data.id);
      break;
  }
};

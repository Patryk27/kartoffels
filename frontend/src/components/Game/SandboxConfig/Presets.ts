export const worlds = [
  {
    id: "small-arena-deathmatch",
    name: "small arena (deathmatch)",
    config: {
      name: "sandbox",
      mode: {
        type: "deathmatch",
      },
      theme: {
        type: "arena",
        radius: 20,
      },
      policy: {
        max_alive_bots: 64,
        max_queued_bots: 128,
      },
    },
  },

  {
    id: "large-arena-deathmatch",
    name: "large arena (deathmatch)",
    config: {
      name: "sandbox",
      mode: {
        type: "deathmatch",
      },
      theme: {
        type: "arena",
        radius: 40,
      },
      policy: {
        max_alive_bots: 128,
        max_queued_bots: 256,
      },
    },
  },

  {
    id: "small-dungeon-deathmatch",
    name: "small dungeon (deathmatch)",
    config: {
      name: "sandbox",
      mode: {
        type: "deathmatch",
      },
      theme: {
        type: "dungeon",
        size: [60, 60],
      },
      policy: {
        max_alive_bots: 64,
        max_queued_bots: 128,
      },
    },
  },

  {
    id: "large-dungeon-deathmatch",
    name: "large dungeon (deathmatch)",
    config: {
      name: "sandbox",
      mode: {
        type: "deathmatch",
      },
      theme: {
        type: "dungeon",
        size: [80, 80],
      },
      policy: {
        max_alive_bots: 128,
        max_queued_bots: 256,
      },
    },
  },
];

export function getTutorialWorld() {
  return {
    name: "tutorial",
    mode: {
      type: "deathmatch",
    },
    theme: {
      type: "arena",
      radius: 16,
    },
    policy: {
      max_alive_bots: 16,
      max_queued_bots: 16,
    },
  };
}

export function getDefaultWorld() {
  return worlds[0].config;
}

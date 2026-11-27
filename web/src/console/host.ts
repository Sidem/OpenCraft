// Connects the admin console's commands (console/commands.ts) to the running game: engine calls for the time, the
// position and items, and the renderer's forced weather. Keeps no state. Add a line here when a command needs a new
// capability (and its member in `ConsoleHost`).

import type { Game } from '../wasm/engine.js';
import type { Renderer } from '../render/renderer';
import type { ConsoleHost } from './commands';

/** `shared` answers whether the world is in a co-op session right now (it can start after the page loads). */
export function consoleHost(game: Game, renderer: Renderer, shared: () => boolean): ConsoleHost {
  return {
    setWeather: (w) => renderer.weather.hold(w),
    weatherOff: () => renderer.skySettings.weather === 0,
    timeOfDay: () => game.time_of_day(),
    dayNumber: () => game.day_number(),
    daySeconds: () => game.day_seconds(),
    skip: (seconds) => game.skip_time(seconds),
    season: () => game.season(),
    dayOfSeason: () => game.day_of_season(),
    daysPerSeason: () => game.days_per_season(),
    weatherHere: () => Array.from(game.weather_here()),
    forecast: (days) => Array.from(game.weather_forecast(days)),
    weatherAhead: (want) => game.weather_ahead(want),
    setTimeSpeed: (speed) => game.set_time_speed(speed),
    timeSpeed: () => game.time_speed(),
    position: () => [game.player_x(), game.player_y(), game.player_z()],
    teleport: (x, y, z) => game.teleport(x, y, z),
    find: (query) => Array.from(game.find_item(query), (id) => ({ id, name: game.item_name(id) })),
    give: (id, count) => game.give(id, count),
    setupDemo: (name) => (name === 'grader' ? game.setup_grader_demo() : `Unknown demo "${name}".`),
    shared: () => shared() || game.is_client(),
  };
}

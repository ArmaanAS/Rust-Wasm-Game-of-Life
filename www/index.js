import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const WIDTH = 512,
  HEIGHT = 512;
let CELL_SIZE = document.body.clientWidth / WIDTH | 0;

const canvas = document.getElementsByTagName("canvas")[0];
canvas.width = CELL_SIZE * WIDTH;
canvas.height = CELL_SIZE * HEIGHT;

const ctx = canvas.getContext("2d");

const universe = Universe.new(WIDTH, HEIGHT, CELL_SIZE);
let canvasPtr = universe.canvas();

// Performance counters
let counter = 0,
  fpsCounter = 0,
  start = performance.now(),
  fpsStart = performance.now(),
  draw_total = 0,
  tick_total = 0,
  canvas_total = 0;
let paused = false,
  running = true;
function drawUniverseCanvas() {
  // let canvas_start = performance.now();
  // const canvasPtr = universe.canvas();
  // canvas_total += performance.now() - canvas_start;

  const data = new Uint8ClampedArray(memory.buffer, canvasPtr,
    WIDTH * HEIGHT * CELL_SIZE * CELL_SIZE * 4);

  const imageData = new ImageData(data, WIDTH * CELL_SIZE);
  ctx.putImageData(imageData, 0, 0);
}

async function renderLoop() {
  if (counter === 0) start = performance.now();
  running = true;

  let start_tick = performance.now();
  universe.tick();
  tick_total += performance.now() - start_tick;

  let start_draw = performance.now()
  drawUniverseCanvas();
  draw_total += performance.now() - start_draw;
  counter++;
  fpsCounter++;

  if (paused)
    running = false;
  else
    requestAnimationFrame(renderLoop);
}


setInterval(() => {
  if (counter === 0) return;

  const elapsed = (performance.now() - start) / 1000;
  console.log("Loops/s:", (counter / elapsed) | 0);
  console.log("Avg draw:", +(draw_total / counter).toFixed(3), "ms");
  console.log("Avg tick:", +(tick_total / counter).toFixed(3), "ms");
  // console.log("Avg canvas:", +(canvas_total / counter).toFixed(3), "ms");
  counter = 0;
  draw_total = 0;
  tick_total = 0;
  canvas_total = 0;
  // start = performance.now();
}, 10000);

const fps = document.getElementById('fps');
setInterval(() => {
  const elapsed = (performance.now() - fpsStart) / 1000;
  fps.innerText = Math.round(fpsCounter / elapsed);
  fpsStart = performance.now();
  fpsCounter = 0;
}, 500)

drawUniverseCanvas();
requestAnimationFrame(renderLoop);
// setInterval(renderLoop);


let drawing = false, wasPaused = false;
let cellState = Cell.Alive;
canvas.addEventListener("pointerdown", e => {
  wasPaused = paused;
  paused = true;
  drawing = true;
  const cx = (e.pageX - canvas.offsetLeft) / CELL_SIZE | 0;
  const cy = (e.pageY - canvas.offsetTop) / CELL_SIZE | 0;

  cellState = universe.get(cx, cy) === Cell.Alive ?
    Cell.Dead : Cell.Alive;
  universe.set(cx, cy, cellState);
});
canvas.addEventListener("pointermove", e => {
  if (drawing) {
    const cx = (e.pageX - canvas.offsetLeft) / CELL_SIZE | 0;
    const cy = (e.pageY - canvas.offsetTop) / CELL_SIZE | 0;

    universe.set(cx, cy, cellState);
    requestAnimationFrame(drawUniverseCanvas);
  }
});
document.addEventListener("pointerup", e => {
  if (drawing) {
    if (!wasPaused) {
      paused = false;
      if (!running)
        requestAnimationFrame(renderLoop);
    }
    drawing = false;
  }
});

const pauseButton = document.getElementById('pause');
const tickButton = document.getElementById('tick');
tickButton.disabled = true;
tick.addEventListener("click", e => {
  if (paused) renderLoop();
})
function pause() {
  if (paused) {
    paused = false;
    if (!running)
      requestAnimationFrame(renderLoop);

    tickButton.disabled = true;
    pauseButton.innerText = "Pause";
  } else {
    paused = true;
    tickButton.disabled = false;
    pauseButton.innerText = "Play";
  }
}
pauseButton.addEventListener("click", e => pause());


let resized = false;
function resize(cell_size, lock = false) {
  resized = lock;
  CELL_SIZE = cell_size;
  canvas.width = CELL_SIZE * WIDTH;
  canvas.height = CELL_SIZE * HEIGHT;
  universe.resize(CELL_SIZE);
  canvasPtr = universe.canvas();

  if (paused)
    drawUniverseCanvas();
}
document.addEventListener("wheel", e => {
  console.log("wheel", e.deltaY)
  if (e.deltaY < 0 && CELL_SIZE < 12)
    resize(CELL_SIZE + 1, true);
  else if (e.deltaY > 0 && CELL_SIZE > 1)
    resize(CELL_SIZE - 1, true);
  else return;
});
window.addEventListener("resize", e => {
  const size = document.body.clientWidth / WIDTH | 0;
  if (!resized && size > 0 && size !== CELL_SIZE)
    resize(size);
}, true)
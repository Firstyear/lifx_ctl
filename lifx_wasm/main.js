import init, { run_app } from './pkg/lifx_wasm.js';
async function main() {
   await init('/pkg/lifx_wasm_bg.wasm');
   run_app();
}
main()

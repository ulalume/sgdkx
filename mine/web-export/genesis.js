/* genesis.js - Placeholder for Genesis Web Emulator JS */

document.addEventListener("DOMContentLoaded", function () {
    const romName = "rom.bin";
    const romNameElem = document.getElementById("rom-name");
    if (romNameElem) {
        romNameElem.textContent = romName;
    }

    // Placeholder: In a real implementation, this would load the WASM emulator and the ROM.
    console.log("Genesis Web Emulator JS loaded. (This is a placeholder.)");
    // Example: loadWasmEmulator('genesis.wasm', romName);
});

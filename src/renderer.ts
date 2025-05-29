/**
 * This file will automatically be loaded by vite and run in the "renderer" context.
 * To learn more about the differences between the "main" and the "renderer" context in
 * Electron, visit:
 *
 * https://electronjs.org/docs/tutorial/process-model
 *
 * By default, Node.js integration in this file is disabled. When enabling Node.js integration
 * in a renderer process, please be aware of potential security implications. You can read
 * more about security risks here:
 *
 * https://electronjs.org/docs/tutorial/security
 *
 * To enable Node.js integration in this file, open up `main.ts` and enable the `nodeIntegration`
 * flag:
 *
 * ```
 *  // Create the browser window.
 *  mainWindow = new BrowserWindow({
 *    width: 800,
 *    height: 600,
 *    webPreferences: {
 *      nodeIntegration: true
 *    }
 *  });
 * ```
 */
import type {Folder, OptParams} from "bindings-folder"
import './index.css';

document.querySelector("#btn_get_folder").addEventListener("click", async (event) => {
    const str = await folderApi.readFolder({path_str: "C:/"});
    console.log(JSON.parse(str));
});
document.querySelector("#btn_set_state").addEventListener("click", async (event) => {
    const dt = new Date().toString();
    const str = await folderApi.setStatus({key: "a", val: {"Text": dt}});
    console.log(JSON.parse(str));
});
document.querySelector("#btn_get_state").addEventListener("click", async (event) => {
    const str = await folderApi.getStatus({key: "a", val: "None"});
    console.log(JSON.parse(str));
});

console.log('👋 This message is being logged by "renderer.ts", included via Vite');

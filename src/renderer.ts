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

import "./resources/fontawesome/css/all.min.css";
import './index.css';
import Split from 'split.js';
import type {OrdItem, OrderBy, OrderAsc } from "bindings-folder"
import {IFolderAPI} from "./preload";
const div_tree: HTMLDivElement = document.querySelector(".tree");
const div_content: HTMLDivElement = document.querySelector(".content");

declare global {
    interface Window {
        api: IFolderAPI
    }
}

Split([div_tree, div_content], {
    sizes: [30, 70],
    minSize: 5,
    gutterSize: 5,
    cursor: 'col-resize',
});

let g_ordering: OrdItem [] = [{nm: "Dir", asc: "Asc"}];
window.addEventListener('DOMContentLoaded', async () => {
    console.log('onload');
    g_ordering = await window.api.getStateObj("ordering", g_ordering);
    console.log(g_ordering);

});


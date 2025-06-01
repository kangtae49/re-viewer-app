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
import type {OrdItem, Folder, MetaType, Item} from "bindings-folder"
import {IFolderAPI} from "./preload";
import {createOptParams} from "./renderer_utils";

const div_tree: HTMLDivElement = document.querySelector(".tree");
const div_content: HTMLDivElement = document.querySelector(".content");

declare global {
    interface Window {
        api: IFolderAPI
    }
}
const api = window.api;

Split([div_tree, div_content], {
    sizes: [30, 70],
    minSize: 5,
    gutterSize: 5,
    cursor: 'col-resize',
});

const g_cache_nm = "folder_cache";
const g_fetch_size = 5000;
const g_tree_order: OrdItem [] = [{nm: "Dir", asc: "Asc"}, {nm: "Nm", asc: "Asc"}];
let g_cur_path: string;
const g_tree_meta: MetaType [] = ["Sz", "Has"];
const g_sep = "\\";

window.addEventListener('DOMContentLoaded', async () => {
    console.log('onload');
    // g_tree_order = await api.getState("ordering", g_tree_order);
    g_cur_path = await api.getCurPath();
    console.log(g_cur_path);

    g_cur_path = "C:\\Windows\\WinSxS"; // TODO: debug
    await render_folder(g_cur_path);

});

const render_folder = async (dir: string) => {

    let folder:Folder = await api.readFolder(createOptParams({
        cache_nm: g_cache_nm,
        path_str: dir,
        ordering: g_tree_order,
        meta_types: g_tree_meta,
        skip_n: 0,
        take_n: g_fetch_size,
    }));
    console.log(0, folder);
    const base_path = [folder.base_nm, folder.item.nm].join(g_sep);
    let base_li = document.querySelector(`.tree li[data-path="${CSS.escape(base_path)}"]`);
    if (!base_li) {
        const ul = document.createElement("ul");
        ul.innerHTML = path_html(folder);
        div_tree.innerHTML = "";
        div_tree.appendChild(ul);
        base_li = div_tree.querySelector(`li`);
    }
    render_items(base_li, folder);

    const tot = folder.tot;
    const tot_pages = Math.ceil(tot / g_fetch_size);
    for (let i = 1; i < tot_pages; i++) {
        folder = await api.readFolder(createOptParams({
            cache_nm: g_cache_nm,
            path_str: dir,
            ordering: g_tree_order,
            meta_types: g_tree_meta,
            skip_n: i * g_fetch_size,
        }));
        render_items(base_li, folder);
    }
}

const render_items = (base_li: Element, folder: Folder) => {
    const frag = document.createDocumentFragment();
    for (const item of folder.item.items) {
        const ul = document.createElement("ul");
        ul.innerHTML = path_html(item);
        frag.appendChild(ul);
    }
    base_li.appendChild(frag);
}

const path_html = (obj: Folder | Item) => {
    let item: Item;
    let folder: Folder = null;
    if ("base_nm" in obj) {
        folder = obj;
        item = folder.item;
    } else {
        item = obj;
    }
    return `
    <li>
        <div class="item">
          <div class="subitem path-icon">
            <i class="fa-solid ${path_icon(item)}"></i>
          </div>
          <div class="subitem path-name">
            ${path_nm(obj)}
          </div>
        </div>
    </li>        
    `;
}
const path_icon = (item: Item): string => {
    return item.dir ? (item.has ? "fa-folder-open" : "fa-folder") : "fa-file";
}

const path_nm = (obj: Folder | Item): string => {
    if ("base_nm" in obj) {
        const folder: Folder = obj;
        return folder.item.nm == "" ? [folder.base_nm, folder.item.nm].join(g_sep) : folder.item.nm;
    } else {
        const item: Item = obj;
        return item.nm;
    }
}

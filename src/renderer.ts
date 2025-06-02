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
import type {OrdItem, Folder, MetaType, Item} from "bindings-folder"
import {IFolderAPI} from "./preload";
import {createOptParams} from "./renderer_utils";

const div_tree: HTMLDivElement = document.querySelector(".tree");
const resizer: HTMLDivElement = document.querySelector(".resizer");

const scroll: HTMLDivElement = document.querySelector(".scroll");
const scroll_inner: HTMLDivElement = document.querySelector(".scroll .inner");
const div_content: HTMLDivElement = document.querySelector(".content");

declare global {
    interface Window {
        api: IFolderAPI
    }
}
const api = window.api;


const g_cache_nm = "folder_cache";
const g_fetch_size = 500;
const g_tree_order: OrdItem [] = [{nm: "Dir", asc: "Asc"}, {nm: "Nm", asc: "Asc"}];
let g_cur_path: string;
const g_tree_meta: MetaType [] = ["Sz", "Has", "Mt"];
const g_sep = "\\";

window.addEventListener('DOMContentLoaded', async () => {
    console.log('onload');
    resizeLayout();
    // g_tree_order = await api.getState("ordering", g_tree_order);
    g_cur_path = await api.getCurPath();
    console.log(g_cur_path);

    // g_cur_path = "C:\\Windows\\WinSxS"; // TODO: debug
    await renderFolder(g_cur_path, "reload");

    div_tree.addEventListener("click", clickEvent);

})

const renderFolder = async (dir: string, click_type: string) => {
    console.log(`renderFolder: ${dir}`);
    let div_target: HTMLDivElement = document.querySelector(`.tree div.item[data-path="${CSS.escape(dir)}"]`);
    const div_items = div_target?.querySelector(".item .items");
    const reload = click_type == "reload" || (click_type == "toggle" && !div_items);

    div_target?.querySelector(".items")?.remove();

    if (reload) {
        console.log("reload");
        let tot_take = 0;
        for (const skip_n of [0, g_fetch_size]) {
            console.log(skip_n);
            const folder: Folder = await api.readFolder(createOptParams({
                cache_nm: g_cache_nm,
                path_str: dir,
                ordering: g_tree_order,
                meta_types: g_tree_meta,
                skip_n: skip_n,
            }));
            const base_path = folder.item.nm == "" ? folder.base_nm : [folder.base_nm, folder.item.nm].join(g_sep);
            if (!div_target) {
                const div_item = item_dom(folder.item, base_path);
                div_tree.appendChild(div_item);
                div_target = div_item;
            }
            render_items(div_target, folder.item.items, base_path);
            tot_take += folder.take_n;
            if (folder.tot == tot_take) {
                break;
            }
        }
    }

    scroll_inner.style.height = div_tree.scrollHeight + 'px';
    if (div_tree.clientHeight == div_tree.scrollHeight) {
        scroll.style.display = "none";
    } else {
        scroll.style.display = "";
    }
}

const render_items = (base_div: Element, items: Item[], base_path: string) => {

    const frag = document.createDocumentFragment();
    const div_items = document.createElement("div");
    div_items.classList.add("items");
    base_div.appendChild(div_items);
    for (const item of items) {
        const div = item_dom(item, base_path);
        frag.appendChild(div);
    }
    div_items.appendChild(frag);
}

const item_dom = (item: Item, base_path: string) => {
    const div = document.createElement("div");
    div.classList.add("item");
    set_dataset(div, item, base_path);

    const nm = item.nm == "" ? base_path : item.nm;
    div.innerHTML = `<div class="label"><i class="fa-solid ${path_icon(item)}"></i>${nm}</div>`;
    return div;
}

const set_dataset = (div: HTMLDivElement, item: Item, base_path: string) => {
    if (item.nm) div.dataset.nm = item.nm;
    if (item.ext) div.dataset.ext = item.ext;
    if (item.dir === true) div.dataset.dir = String(item.dir);
    if (item.mt) div.dataset.mt = item.mt;
    if (item.cnt) div.dataset.cnt = String(item.cnt);
    if (item.has === true) div.dataset.has = String(item.has);
    if (item.sz) div.dataset.sz = String(item.sz);
    if (item.tm) div.dataset.tm = String(item.tm);
    div.dataset.path = [base_path, item.nm].join(g_sep);
    div.title = item.nm;
}

const path_icon = (item: Item): string => {
    return item.dir ? (item.has ? "fa-folder-open" : "fa-folder") : "fa-file";
}
//
// const path_base_nm = (folder: Folder): string => {
//     return folder.item.nm == "" ? [folder.base_nm, folder.item.nm].join(g_sep) : folder.item.nm;
// }

// const path_nm = (item: Item): string => {
//     return item.nm;
// }


let isDragging = false;

resizer.addEventListener('mousedown', () => {
    isDragging = true;
    const overlay: HTMLDivElement = document.querySelector(".iframe-overlay");
    if (overlay) {
        overlay.classList.add("active");
    }
    document.body.style.cursor = 'ew-resize';
});

document.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    resizeLayout(e.clientX);
});

// let mouse_x;
document.addEventListener('mouseup', () => {
    isDragging = false;
    const overlay: HTMLDivElement = document.querySelector(".iframe-overlay");
    if (overlay) {
        overlay.classList.remove("active");
    }
    document.body.style.cursor = '';
});

scroll.addEventListener('scroll', () => {
    if (div_tree.scrollTop != scroll.scrollTop) {
        div_tree.scrollTop = scroll.scrollTop;
    }
});

div_tree.addEventListener('scroll', () => {
    if (div_tree.scrollTop != scroll.scrollTop) {
        scroll.scrollTop = div_tree.scrollTop;
    }
});



window.addEventListener('resize', () => {
    resizeLayout();
});

const resizeLayout = (left: number | undefined = undefined) => {
    const default_left = 200;
    const resizer_width = 6;
    const scroll_width = 15;

    if (left === undefined) {
        left = resizer.offsetLeft;
    }
    if (left == 0) {
        left = default_left;  // default
    }

    const minLeft = 0;
    const maxLeft = window.innerWidth; // - 100;
    const resizerLeft = Math.min(Math.max(left, minLeft), maxLeft);
    const contentLeft = resizerLeft + resizer_width;
    const contentWidth = window.innerWidth - contentLeft;

    resizer.style.left = resizerLeft + 'px';
    scroll.style.left = (resizerLeft - scroll_width) + 'px';
    div_content.style.left = contentLeft + 'px';
    div_content.style.width = contentWidth + 'px';

    if (div_tree.clientHeight == div_tree.scrollHeight) {
        scroll.style.display = "none";
    } else {
        scroll.style.display = "";
    }
}

const clickEvent = async (e: Event) => {
    const target = e.target as HTMLDivElement;
    const tagName = target.tagName;
    const div_item = target.closest(".item") as HTMLDivElement;
    if (tagName == "DIV" && target.classList.contains('item')) {
        const mouseEvent = e as MouseEvent;
        if (mouseEvent.offsetX < 0) {
            const parentItem: HTMLDivElement = target.closest('.items').closest('.item');
            parentItem.querySelector("i").click();
        }
        return;
    }

    const dataset = div_item.dataset;
    if (dataset.dir && tagName == "I") {  // click dir icon
        console.log(dataset.path);
        await renderFolder(dataset.path, "toggle");
    } else if (dataset.dir && tagName == "DIV") {  // click dir label

    } else if (!dataset.dir && tagName == "I") {
        await viewFile(div_item);
    } else if (!dataset.dir && tagName == "DIV" && target.classList.contains('label')) {  // click file label
        await viewFile(div_item);
    } else {
        return;
    }
}

const viewFile = async (div_item: HTMLDivElement) => {
    const dataset = div_item.dataset;
    div_content.innerHTML = "";
    if (dataset.mt.startsWith("image/")) {
        embedImage("image", dataset);
    } else if (dataset.mt.endsWith("/pdf")) {
        embedPdf("pdf", dataset);
    } else if (dataset.mt.endsWith("/html")) {
        embedHtml("html", dataset);
    } else if (dataset.mt.endsWith("/json")) {
        embedIframe("iframe", dataset);
    } else if (dataset.mt.startsWith("audio/") && Number(dataset.sz) > 1024*500) {
        embedVideo("audio", dataset);
    } else if (dataset.mt.startsWith("video/") && Number(dataset.sz) > 1024*500) {
        embedVideo("video", dataset);
    } else if (Number(dataset.sz) < 1024*500) {
        embedHtml("html", dataset);
        // embedIframe("iframe", dataset);
    } else {
        embedHtml("html", dataset);
        // embedIframe("iframe", dataset);
    }
}

const embedImage = (nm: "image", dataset: DOMStringMap) => {
    div_content.dataset.type = nm;
    const div_img = document.createElement("img");
    div_img.src = dataset.path;
    div_img.title = dataset.path;
    div_content.appendChild(div_img);
}

const embedPdf = (nm: "pdf", dataset: DOMStringMap) => {
    div_content.dataset.type = nm;
    const div_embed = document.createElement("embed");
    div_embed.src = dataset.path;
    div_embed.title = dataset.path;
    div_embed.type = dataset.mt;
    div_content.appendChild(div_embed);
}

const embedVideo = (nm: "audio" | "video", dataset: DOMStringMap) => {
    div_content.dataset.type = nm;
    const div_embed = document.createElement(nm);
    div_embed.controls = true;
    div_embed.autoplay = true;
    div_embed.volume = 0.5;
    const source = document.createElement("source");
    source.src = dataset.path;
    source.type = dataset.mt;
    div_embed.appendChild(source);
    div_content.appendChild(div_embed);
}
const embedIframe = (nm: "iframe", dataset: DOMStringMap) => {
    div_content.dataset.type = nm;
    const div_embed = document.createElement(nm);
    div_embed.src = dataset.path;

    const div_overlay = document.createElement("div");
    div_overlay.classList.add("iframe-overlay");
    div_content.appendChild(div_overlay);
    div_content.appendChild(div_embed);
}

const embedHtml = (nm: "html", dataset: DOMStringMap) => {
    div_content.dataset.type = nm;
    api.readText(dataset.path)
        .then((textContent) => {
            console.log(textContent.mimetype, textContent.enc);
            div_content.innerHTML = textContent.text;
        })
        .catch((reason) => {
            console.log(reason);
            div_content.classList.add("html");
            div_content.innerHTML = reason;
        })
}

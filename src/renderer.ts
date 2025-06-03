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
import type {OrdItem, Folder, MetaType, Item} from "../napi-folder/bindings"
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
const g_tree_meta: MetaType [] = ["Sz", "Has", "Mt"];
const g_sep = "\\";
let g_cur_path: string;

window.addEventListener('DOMContentLoaded', async () => {
    console.log('onload');
    resizeLayout();
    // g_tree_order = await api.getState("ordering", g_tree_order);
    try {
        g_cur_path = await api.getCurPath();
    } catch (e) {
        console.error(e);
    }
    console.log(g_cur_path);

    // g_cur_path = "C:\\Windows\\WinSxS"; // TODO: debug

    // g_cur_path = "C:\\"; // TODO: debug

    await renderFolder(g_cur_path, "reload");
    updateSelectedPath(g_cur_path);
    div_tree.addEventListener("click", clickEvent);

})

const renderFolder = async (dir: string, click_type: string) => {
    let div_target: HTMLDivElement = document.querySelector(`.tree div.item[data-path="${CSS.escape(dir)}"]`);
    const div_items = div_target?.querySelector(".item .items");
    const reload = click_type == "reload" || (click_type == "toggle" && !div_items);

    div_target?.querySelector(".items")?.remove();

    if (reload) {
        console.log("reload");
        let tot_take = 0;
        for (const skip_n of [0, g_fetch_size]) {
            const folder: Folder = await api.readFolder(createOptParams({
                cache_nm: g_cache_nm,
                path_str: dir,
                ordering: g_tree_order,
                meta_types: g_tree_meta,
                skip_n: skip_n,
            }));
            const base_path = folder.base_nm;
            if (!div_target) {
                const div_item = item_dom(folder.item, base_path);
                div_tree.appendChild(div_item);
                div_target = div_item;
            }
            const base_path_for_items = folder.item.nm == "" ? base_path : [base_path, folder.item.nm].join(g_sep);
            render_items(div_target, folder.item.items, base_path_for_items);
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
    div_items.title = base_path;
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

    const nm = item.nm == "" ? [base_path, item.nm].join(g_sep) : item.nm;
    div.innerHTML = `<div class="label"><i class="fa-solid ${path_icon(item)}"></i>${nm}</div>`;
    (div.querySelector(".label") as HTMLDivElement).title = nm;
    div.querySelector("i").dataset.title = nm;
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
}

const path_icon = (item: Item): string => {
    return item.dir ? (item.has ? "fa-folder-open" : "fa-folder") : "fa-file";
}


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
    const dataset = div_item?.dataset;
    if (!dataset) return;
    e.preventDefault();

    if (tagName == "DIV" && target.classList.contains('items')) {
        const mouseEvent = e as MouseEvent;
        if (mouseEvent.offsetX < 0) {
            const parentItem: HTMLDivElement = target.closest('.item');
            parentItem.querySelector("i").click();
            updateSelectedPath(parentItem, "center");
        }
        return;
    }

    updateSelectedPath(div_item);
    if (dataset.dir && tagName == "I") {  // click dir icon
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
        viewImg(dataset);
    } else if (dataset.mt.endsWith("/pdf")) {
        viewEmbed(dataset);
    } else if (dataset.mt.endsWith("/html")) {
        viewHtml(dataset);
    } else if (dataset.mt.endsWith("/json")) {
        viewIframe(dataset);
    } else if (dataset.mt.endsWith("/xml")) {
        viewText(dataset);
    } else if (dataset.mt.startsWith("audio/") && Number(dataset.sz) > 1024*500) {
        viewMedia(dataset);
    } else if (dataset.mt.startsWith("video/") && Number(dataset.sz) > 1024*500) {
        viewMedia(dataset);
    } else if (Number(dataset.sz) < 1024*500) {
        viewText(dataset);
    } else {
        viewNone(dataset);
    }
}

const viewImg = (dataset: DOMStringMap) => {
    div_content.dataset.type = "img";
    div_content.dataset.mt = dataset.mt;
    const div_img = document.createElement("img");
    div_img.src = dataset.path;
    div_img.title = dataset.path;
    div_content.appendChild(div_img);
}

const viewEmbed = (dataset: DOMStringMap) => {
    div_content.dataset.type = "embed";
    div_content.dataset.mt = dataset.mt;
    const div_embed = document.createElement("embed");
    div_embed.src = dataset.path;
    div_embed.title = dataset.path;
    div_embed.type = dataset.mt;

    const div_overlay = document.createElement("div");
    div_overlay.classList.add("iframe-overlay");
    div_content.appendChild(div_overlay);

    div_content.appendChild(div_embed);
}

const viewMedia = (dataset: DOMStringMap) => {
    div_content.dataset.type = "media";
    div_content.dataset.mt = dataset.mt;
    const nm = dataset.mt.split("/")[0];
    let div_embed;
    if (nm == "audio") {
        div_embed = document.createElement("audio");
    } else {
        div_embed = document.createElement("video");
    }
    div_embed.controls = true;
    div_embed.autoplay = true;
    div_embed.volume = 0.5;
    const source = document.createElement("source");
    source.src = dataset.path;
    source.type = dataset.mt;
    div_embed.appendChild(source);
    div_content.appendChild(div_embed);
}
const viewIframe = (dataset: DOMStringMap) => {
    div_content.dataset.type = "iframe";
    div_content.dataset.mt = dataset.mt;
    const div_embed = document.createElement("iframe");
    div_embed.src = dataset.path;

    const div_overlay = document.createElement("div");
    div_overlay.classList.add("iframe-overlay");
    div_content.appendChild(div_overlay);
    div_content.appendChild(div_embed);
}

const viewHtml = (dataset: DOMStringMap) => {
    div_content.dataset.type = "html";
    div_content.dataset.mt = dataset.mt;
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

const viewText = (dataset: DOMStringMap) => {
    div_content.dataset.type = "text";
    div_content.dataset.mt = dataset.mt;
    api.readText(dataset.path)
        .then((textContent) => {
            console.log(textContent.mimetype, textContent.enc);
            div_content.innerText = textContent.text;
        })
        .catch((reason) => {
            console.log(reason);
            div_content.classList.add("text");
            div_content.innerText = reason;
        })
}

const viewNone = (dataset: DOMStringMap) => {
    console.log(dataset.mt);
    div_content.dataset.type = "none";
    div_content.innerHTML = "";
}

const updateSelectedPath = (target: string | HTMLDivElement, pos: ScrollLogicalPosition = "nearest") => {
    const cur_selected: HTMLDivElement = div_tree.querySelector(".item.selected");
    let new_selected: HTMLDivElement;
    if (typeof target == "string") {
        new_selected = div_tree.querySelector(`.item[data-path="${CSS.escape(target)}"]`);
    } else {
        new_selected = target;
    }
    if (cur_selected == new_selected) return;
    cur_selected?.classList.remove("selected");
    new_selected?.classList.add("selected");
    new_selected.scrollIntoView({ behavior: 'auto', block: pos, inline: 'end' });
}
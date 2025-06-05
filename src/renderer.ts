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
import './splitter.css';
import {Splitter} from "./splitter";
import type {OrdItem, Folder, MetaType, Item , DiskInfo } from "../napi-folder/bindings"
import {IFolderAPI} from "./preload";
import {SEP, isVisibleInViewport, shadowHtml, setDataset, getDataset} from "./renderer_utils";

const div_tree: HTMLDivElement = document.querySelector(".tree");
const div_left_top: HTMLDivElement = document.querySelector(".left .top");

const div_content: HTMLDivElement = document.querySelector(".content");
const div_title_path: HTMLDivElement = document.querySelector(".title-path");

declare global {
    interface Window {
        api: IFolderAPI
    }

    interface Element {
        getDataset (this: Element): DOMStringMap;
        setDataset (this: Element, obj: Record<string, string | number | boolean | bigint | undefined>): void;
    }
}

Element.prototype.getDataset = getDataset;
Element.prototype.setDataset = setDataset;

type FolderRenderType = "Root" | "Fold" | "Toggle" ;
const api = window.api;

const g_cache_nm = "folder_cache";
const g_fetch_size = 500;
const g_tree_order: OrdItem [] = [{nm: "Dir", asc: "Asc"}, {nm: "Nm", asc: "Asc"}];
const g_tree_meta: MetaType [] = ["Sz", "Has", "Mt"];
let g_cur_path: string;
let g_splitter:  Splitter;


window.addEventListener('DOMContentLoaded', async () => {
    console.log('onload');
    await viewHomeDir();

    g_splitter = new Splitter({
        container: ".main.container",
        targetA: ".left",
        targetB: ".right",
        targetScroll: ".left.container .tree",
        defaultLeft: 200,
    });

    // splitter.resizeLayout();
    // g_tree_order = await api.getState("ordering", g_tree_order);
    try {
        g_cur_path = await api.getCurPath();
    } catch (e) {
        console.error(e);
    }
    console.log(g_cur_path);

    // g_cur_path = "C:\\Windows\\WinSxS"; // TODO: debug
    // g_cur_path = "C:\\"; // TODO: debug
    // g_cur_path = "C:\\sources\\ui\\readme.txt"; // TODO: debug
    // await renderDisks();
    await renderFullPath(g_cur_path);
    // await renderFolder(g_cur_path, "Root");
    div_tree.focus();
    div_tree.addEventListener("click", clickTreeEvent);
    div_tree.addEventListener("keydown", keydownTreeEvent);
    div_left_top.addEventListener("click", clickLeftTopEvent);
    div_title_path.addEventListener("click", clickTitlePathEvent);

})

const renderFolder = async (dir: string, render_type: FolderRenderType = "Root" ) => {

    let div_target: HTMLDivElement = document.querySelector(`.tree div.item[data-path="${CSS.escape(dir)}"]`);
    const div_items = div_target?.querySelector(".item .items");
    const hasChildren = !!div_items;
    div_target?.querySelector(".items")?.remove();

    if (render_type === "Root") {
        div_tree.innerHTML = "";
        div_target = null;
    }
    if (render_type == "Root" || render_type == "Fold" ||
        (render_type == "Toggle" && !hasChildren)) {
        console.log("reload");

        let tot_take = 0;
        for (const skip_n of [0, g_fetch_size]) {
            const folder: Folder = await api.readFolder({
                cache_nm: g_cache_nm,
                path_str: dir,
                ordering: g_tree_order,
                meta_types: g_tree_meta,
                skip_n: skip_n,
            });
            const base_path = folder.base_nm;
            if (!div_target) {
                const div_item = itemDom(folder.item, base_path);
                div_tree.appendChild(div_item);
                div_target = div_item;
            }
            const base_path_for_items = folder.item.nm == "" ? base_path : [base_path, folder.item.nm].join(SEP);
            renderItems(div_target, folder.item.items, base_path_for_items);
            tot_take += folder.take_n;
            if (folder.tot == tot_take) {
                break;
            }
        }
    }
    g_splitter.resizeLayout();
    updateSelectedPath(div_target);

}

const renderDisks = async () => {
    const disks = await api.getDisks();
    div_tree.innerHTML = "";
    disks.forEach((disk: DiskInfo) => {
        const div_item = itemDom({
            nm: disk.path,
            dir: true
        });
        div_tree.appendChild(div_item);
    })
}

const renderFullPath = async (full_path: string, n = 0) => {
    const arr = full_path.split(SEP);
    if ( n == arr.length - 1) { // last selection
        let selected_path = arr.join(SEP);
        if (selected_path.endsWith(":")) {
            selected_path += SEP;
        }
        const selected = updateSelectedPath(selected_path);
        requestAnimationFrame(() => {
            const label: HTMLDivElement = selected?.querySelector(".label");
            label?.click();
        });
        return;
    }
    let render_type: FolderRenderType;
    if (n == 0){
        await renderDisks();
        // render_type = "Root";
        render_type = "Fold";
    } else {
        render_type = "Fold";
    }
    let path = arr.slice(0, n+1).join(SEP);
    if (path.endsWith(":")) {
        path += SEP;
    }
    console.log(path);
    await renderFolder(path, render_type);
    await renderFullPath(full_path, n+1);
}


const renderItems = (base_div: Element, items: Item[], base_path: string) => {

    const frag = document.createDocumentFragment();
    const div_items = document.createElement("div");
    div_items.classList.add("items");
    div_items.title = base_path;
    base_div.appendChild(div_items);
    for (const item of items) {
        const div = itemDom(item, base_path);
        frag.appendChild(div);
    }
    div_items.appendChild(frag);
}

const itemDom = (item: Item, base_path?: string) => {
    const div = document.createElement("div");
    div.classList.add("item");
    let path: string;
    if (base_path) {
        path = [base_path, item.nm].join(SEP);
    } else {
        path = item.nm;
    }
    div.setDataset({
        nm: item?.nm,
        dir: item?.dir,
        ext: item?.ext,
        mt: item?.mt,
        cnt: item?.cnt,
        has: item?.has,
        sz: item?.sz,
        tm: item?.tm,
        path: path
    });

    const nm = item.nm == "" ? [base_path, item.nm].join(SEP) : item.nm;
    div.innerHTML = `<div class="label"><i class="fa-solid ${pathIcon(item)}"></i>${nm}</div>`;
    const div_label: HTMLDivElement = div.querySelector(".label");
    div_label.title = nm;
    div.querySelector("i").setDataset({title: nm});
    return div;
}


const pathIcon = (item: Item): string => {
    return item.dir ? (item.has ? "fa-folder-plus" : "fa-folder") : "fa-file";
}


const clickTreeEvent = async (e: Event) => {
    const target = e.target as HTMLDivElement;
    const tagName = target.tagName;
    const div_item = target.closest(".item");
    const dataset = div_item?.getDataset();
    if (!dataset) return;
    e.preventDefault();

    if (tagName == "DIV" && target.classList.contains('items')) {
        const mouseEvent = e as MouseEvent;
        if (mouseEvent.offsetX < 0) {
            const parentItem: HTMLDivElement = target.closest('.item');
            updateSelectedPath(parentItem);
            parentItem.querySelector("i").click();
        }
        return;
    }

    if (target.classList.contains('item')) {
        return;
    }

    updateSelectedPath(div_item);
    if (dataset.dir && tagName == "I") {  // click dir icon
        await renderFolder(dataset.path, "Toggle");
    } else if (dataset.dir && tagName == "DIV") {  // click dir label
    } else if (!dataset.dir && tagName == "I") {
        await viewFile(div_item);
    } else if (!dataset.dir && tagName == "DIV" && target.classList.contains('label')) {  // click file label
        await viewFile(div_item);
    } else {
        return;
    }

}

const viewFile = async (div_item: Element) => {
    const dataset = div_item.getDataset();
    updateContentTitle(dataset);
    div_content.innerHTML = "";
    if (dataset.mt.startsWith("image/")) {
        viewImg(dataset);
    } else if (dataset.mt.endsWith("/pdf")) {
        viewEmbed(dataset);
    } else if (dataset.mt.endsWith("/html")) {
        viewShadowHtml(dataset);
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
    div_content.setDataset({
        type: "img",
        mt: dataset.mt
    });
    const div_img  = document.createElement("img");
    div_img.src = dataset.path;
    div_img.title = dataset.path;
    div_content.appendChild(div_img);
}

const viewEmbed = (dataset: DOMStringMap) => {
    div_content.setDataset({
        type: "embed",
        mt: dataset.mt,
    })
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
    div_content.setDataset({
        type: "media",
        mt: dataset.mt,
    })
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
    div_content.setDataset({
        type: "iframe",
        mt: dataset.mt,
    })
    const div_embed = document.createElement("iframe");
    div_embed.src = dataset.path;

    const div_overlay = document.createElement("div");
    div_overlay.classList.add("iframe-overlay");
    div_content.appendChild(div_overlay);
    div_content.appendChild(div_embed);
}

/*
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
            div_content.innerHTML =reason;
        })
}
 */

const viewShadowHtml = (dataset: DOMStringMap) => {
    div_content.setDataset({
        type: "html",
        mt: dataset.mt,
    })
    api.readText(dataset.path)
        .then((textContent) => {
            console.log(textContent.mimetype, textContent.enc);
            shadowHtml(div_content, textContent.text);
        })
        .catch((reason) => {
            console.log(reason);
            shadowHtml(div_content, reason);
        })
}

const viewText = (dataset: DOMStringMap) => {
    div_content.setDataset({
        type: "text",
        mt: dataset.mt,
    })
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
    div_content.setDataset({
        type: "none",
        mt: dataset.mt,
    })
    div_content.innerHTML = "";
}

const updateSelectedPath = (target: string | Element, pos: ScrollLogicalPosition | null = "center"): Element => {
    const cur_selected = div_tree.querySelector(".item.selected");
    let new_selected: Element;
    if (typeof target == "string") {
        new_selected = div_tree.querySelector(`.item[data-path="${CSS.escape(target)}"]`);
    } else {
        new_selected = target;
    }
    cur_selected?.classList.remove("selected");
    new_selected?.classList.add("selected");
    if(pos) {
        if (!isVisibleInViewport(new_selected, div_tree)) {
            scrollIntoView(new_selected?.querySelector(".label"));
            new_selected?.querySelector(".label").scrollIntoView({ behavior: 'auto', block: pos, inline: 'end' });
            console.log(`scrollIntoView ${pos}`);
        }
    }
    return new_selected;
}

const scrollIntoView = (div: HTMLDivElement, pos: ScrollLogicalPosition = "center") => {
    div?.scrollIntoView({ behavior: 'auto', block: pos, inline: 'end' });
}

const keydownTreeEvent = async (e: Event) => {
    const keys = ["Enter", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", " "];
    const key = (e as KeyboardEvent).key;
    if(keys.includes(key)){
        e.stopPropagation();
        e.preventDefault();
    } else {
        return;
    }
    const item_selected: HTMLDivElement = div_tree.querySelector(".item.selected");
    if (key === "Enter") {
        if (item_selected.dataset.dir) {
            item_selected.querySelector("i").click();
        } else {
            (item_selected.querySelector(".label") as HTMLElement).click();
        }
    } else if (key === "ArrowUp") {
        const arr = Array.from(div_tree.querySelectorAll(".item"));
        const idx = arr.indexOf(item_selected);
        const prev_idx = Math.max(idx-1, 0);
        // if (idx != prev_idx) {
            updateSelectedPath(arr[prev_idx]);
        // }
    } else if (key === "ArrowDown") {
        const arr = Array.from(div_tree.querySelectorAll(".item"));
        const idx = arr.indexOf(item_selected);
        const next_idx = Math.min(idx+1, arr.length-1);
        // if (idx != next_idx) {
            updateSelectedPath(arr[next_idx]);
        // }
    } else if (key === "ArrowLeft") {
        const parent = item_selected.closest(".items").closest(".item");
        const has_items = item_selected.querySelector(".item") != null;
        if (item_selected.dataset.dir && has_items) {
            item_selected?.querySelector("i").click();

            const new_item_selected: HTMLDivElement = div_tree.querySelector(".item.selected");
            updateSelectedPath(new_item_selected);
        } else {
            parent?.querySelector("i").click();
            updateSelectedPath(parent);
        }
    } else if (key === "ArrowRight") {
        if (item_selected.dataset.dir) {
            item_selected?.querySelector("i").click();
        } else {
            item_selected?.querySelector("i").click();
        }
        updateSelectedPath(item_selected);

    } else if (key === " ") {
        (item_selected?.querySelector(".label") as HTMLDivElement).click();
    }

}

const updateContentTitle = (dataset: DOMStringMap) => {
    const cur_dir_all = document.querySelector(".cur-dir .dir-all");
    const arr_dir = dataset.path.split(SEP);
    cur_dir_all.innerHTML = "";
    for(let i=0; i<arr_dir.length; i++) {
        if (i != 0) {
            cur_dir_all.append(SEP);
        }
        let path = arr_dir.slice(0, i+1).join(SEP);
        if (path.endsWith(":")) {
            path += SEP;
        }
        const div_part = document.createElement("div");
        div_part.classList.add("dir-part");
        div_part.innerHTML = arr_dir[i];
        div_part.dataset.path = path;
        cur_dir_all.appendChild(div_part);
    }
}

const viewHomeDir = async () => {
    const homeDir = await api.getHomeDir();
    console.log(homeDir);
    div_left_top.querySelector(".link.root").setDataset({path: "/"});
    div_left_top.querySelector(".link.home").setDataset({path: homeDir.HomeDir});
    div_left_top.querySelector(".link.down").setDataset({path: homeDir.DownloadDir});
    div_left_top.querySelector(".link.docs").setDataset({path: homeDir.DocumentDir});
    div_left_top.querySelector(".link.video").setDataset({path: homeDir.VideoDir});
    div_left_top.querySelector(".link.music").setDataset({path: homeDir.AudioDir});
    div_left_top.querySelector(".link.image").setDataset({path: homeDir.PictureDir});
    div_left_top.querySelector(".link.desktop").setDataset({path: homeDir.DesktopDir});
}

const clickLeftTopEvent = async (e: Event) => {
    const target = e.target as HTMLDivElement;
    if (target.tagName != "I") {
        return;
    }
    e.preventDefault();

    const link: HTMLDivElement = target.closest(".link");
    let path = link.dataset.path;
    if (path == "/") {
        renderDisks();
        return;
    } else if (path == ".") {
        const div_item: HTMLDivElement = div_tree.querySelector(".item");
        path = div_item.dataset.path;
    } else if (path == "..") {
        const div_item: HTMLDivElement = div_tree.querySelector(".item");
        path = div_item.dataset.path;
        if (!path.endsWith(SEP)) {
            const div_item: HTMLDivElement = div_tree.querySelector(".item");
            const arr = div_item.dataset.path.split(SEP);
            path = arr.slice(0, -1).join(SEP);
        }
    }
    console.log("top:", path);
    // await renderFolder(path, "Root");
    await renderFullPath(path);
}

const clickTitlePathEvent = async (e: MouseEvent) => {
    const target = e.target as HTMLDivElement;
    const dir_part: HTMLDivElement = target.closest(".dir-part");
    if(dir_part?.dataset?.path) {
        await renderFullPath(dir_part.dataset.path);
    }
}

// TODO: drive list
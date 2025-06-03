// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts


import {contextBridge, ipcRenderer} from 'electron'
import path from "path";
import type {Folder, OptParams, TextContent} from "../napi-folder/bindings"

// __dirname: re-viewer-app\.vite\build
const isDev = process.env.NODE_ENV === "development";
const nativePath = isDev
    ? path.join(__dirname, "../../napi-folder")
    : path.join(process.resourcesPath, "napi-folder");
// eslint-disable-next-line @typescript-eslint/no-var-requires
const {FolderApi} = require(nativePath);


export interface IFolderAPI {
    getCurPath: () => Promise<string>,
    readFolder: (params: OptParams) => Promise<Folder>,
    readText: (pathStr: string) => Promise<TextContent>,
    setState: <T> (key: string, val: T) => Promise<T>,
    getState: <T> (key: string, default_val: object | undefined) => Promise<T>,
}

const api: IFolderAPI = {
    getCurPath: async (): Promise<string> => {
        return await ipcRenderer.invoke('get-cur-path');
    },
    readFolder: async (params: OptParams): Promise<Folder> => {
        return (new FolderApi()).readFolder(JSON.stringify(params))
            .then(JSON.parse)
    },
    readText: async (pathStr: string): Promise<TextContent> => {
        return (new FolderApi()).readText(pathStr)
            .then(JSON.parse)
    },
    setState: async <T>(key: string, val: T): Promise<T> => {
        let str_val;
        if (val !== null && typeof val === 'object') {
            str_val = JSON.stringify(val)
        } else {
            str_val = val;
        }

        const new_val = await (new FolderApi()).setState(key, str_val);
        if (val !== null && typeof val === 'object') {
            return JSON.parse(new_val)
        } else {
            return new_val
        }
    },
    getState: async <T>(key: string, default_val: object | undefined): Promise<T> => {
        let obj_default_val;
        if (default_val != null && typeof default_val === 'object') {
            obj_default_val = JSON.stringify(default_val);
        }
        return (new FolderApi()).getState(key, obj_default_val)
            .then(JSON.parse);
    },

};


contextBridge.exposeInMainWorld('api', api);



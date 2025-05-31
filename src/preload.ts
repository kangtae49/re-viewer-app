// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts


import {contextBridge} from 'electron'
import type {Folder, OptParams} from "bindings-folder"

// eslint-disable-next-line @typescript-eslint/no-var-requires
const {FolderApi} = require("napi-folder")

// export type StateObj = {
//     key: string
//     val: any
// }
export interface IFolderAPI {
    readFolder: (params: OptParams) => Promise<Folder>,
    readText: (pathStr: string) => Promise<string>,
    setState: <T> (key: string, val: T) => Promise<T>,
    getState: (key: string, default_val: string | undefined | null) => Promise<string>,
    getStateObj: <T> (key: string, default_val: object | undefined) => Promise<T>,
}

const api: IFolderAPI = {
    readFolder: async (params: OptParams): Promise<Folder> => {
        return (new FolderApi()).readFolder(JSON.stringify(params))
            .then(JSON.parse)
    },
    readText: async (pathStr: string): Promise<string> => {
        return (new FolderApi()).readText(pathStr)
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
    getState: async (key: string, default_val: string | undefined | null): Promise<string> => {
        return (new FolderApi()).getState(key, default_val);
    },
    getStateObj: async <T>(key: string, default_val: object | undefined): Promise<T> => {
        let obj_default_val;
        if (default_val != null && typeof default_val === 'object') {
            obj_default_val = JSON.stringify(default_val);
        }
        return (new FolderApi()).getState(key, obj_default_val)
            .then(JSON.parse);
    },

};


contextBridge.exposeInMainWorld('api', api);



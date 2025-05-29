// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts
import {contextBridge, ipcRenderer} from 'electron'

// eslint-disable-next-line @typescript-eslint/no-var-requires
const {FolderApi} = require("napi-folder")
import type {OptParams} from "bindings-folder"

contextBridge.exposeInMainWorld('folderApi', {
    // desktop: true,
    // setTitle: (title: string): void => ipcRenderer.send('set-title', title),
    readFolder: (params: OptParams) => {
        // const p = JSON.stringify(params);
        // console.log(p);
        return (new FolderApi()).readFolder(JSON.stringify(params))
    },
})
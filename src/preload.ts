// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts
import {contextBridge, ipcRenderer} from 'electron'

// eslint-disable-next-line @typescript-eslint/no-var-requires
const {FolderApi} = require("napi-folder")
import type {OptParams, StateParams} from "bindings-folder"

contextBridge.exposeInMainWorld('folderApi', {
    // desktop: true,
    // setTitle: (title: string): void => ipcRenderer.send('set-title', title),
    readFolder: (params: OptParams): Promise<string> => {
        return (new FolderApi()).readFolder(JSON.stringify(params))
    },
    setStatus: (params: StateParams): Promise<string> => {
        return (new FolderApi()).setState(JSON.stringify(params))
    },
    getStatus: (params: StateParams): Promise<string> => {
        return (new FolderApi()).getState(JSON.stringify(params))
    },
})
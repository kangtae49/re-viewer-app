# Electron & Typescript & napi-rs

```sh
npx create-electron-app@latest re-viewer-app --template=vite-typescript
cd re-viewer-app
npm install
npm run start

```

## NAPI 
re-viewer-app/package.json

```json
{
  "scripts": {
    "start": "electron ."
  },
  "dependencies": {
    "napi-folder": "./napi-folder",
    "bindings-folder": "./napi-folder/bindings"
  }
}
```

main.js
```js
const mainWindow = new BrowserWindow({
    webPreferences: {
        preload: path.join(__dirname, 'preload.js'),
        sandbox: false,  // for napi-folder
    },
});
```

## publish
```sh
npm run make
npm run publish
```

```js
await folderApi.readFolder({path_str: "C:/"})
await folderApi.setStatus({key: "a", val: {"Text": "------------" }})
await folderApi.getStatus({key: "a", val: "None"})

```
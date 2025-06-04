# Electron & Typescript & napi-rs

```sh
npx create-electron-app@latest re-viewer-app --template=vite-typescript
cd re-viewer-app
npm install
npm run start

```

## NAPI 
re-viewer-app/forge.config.ts
```ts
const config: ForgeConfig = {
  packagerConfig: {
    asar: true,
    extraResource: [
        './napi-folder/index.js',
        './napi-folder/index.d.ts',
        './napi-folder/napi-folder.win32-x64-msvc.node',
        './napi-folder/package.json',
    ],
  }
 }
```

re-viewer-app/src/preload.js
```ts
import type {Folder, OptParams, TextContent} from "../napi-folder/bindings"

// __dirname: re-viewer-app\.vite\build
const isDev = process.env.NODE_ENV === "development";
const nativePath = isDev
? path.join(__dirname, "../../napi-folder")
: path.join(process.resourcesPath, "napi-folder");
// eslint-disable-next-line @typescript-eslint/no-var-requires
const {FolderApi} = require(nativePath);
```

re-viewer-app/src/main.js
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


import path from 'node:path';
import { stat } from 'fs/promises';

export const getCurPath = async (argv: string[], isPackaged: boolean): Promise<string> => {
    const args = argv.slice(isPackaged ? 1 : 2);
    let path_str = ".";
    if (args.length > 0) {
        path_str = args[0];
    }
    const absolutePath = path.resolve(path_str);

    const stats = await stat(absolutePath);
    let basePath: string;
    if (stats.isDirectory()){
        const nm = path.basename(absolutePath);
        const dirname = path.dirname(absolutePath);
        if (nm == "") {

            basePath = dirname;
        } else {
            basePath = [dirname, nm].join(path.sep);
        }
    } else {
        basePath = path.dirname(absolutePath);
    }
    return basePath;
};



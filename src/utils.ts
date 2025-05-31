import path from 'node:path';
import { stat } from 'fs/promises';

export const getCurPath = async (argv: string[], isPackaged: boolean): Promise<string> => {
    const args = argv.slice(isPackaged ? 1 : 2);
    let path_str = ".";
    if (args.length > 0) {
        path_str = args[0];
    }
    console.log(args.length);
    console.log(path_str);
    const stats = await stat(path_str);
    if (stats.isDirectory()){
        return path_str;
    } else {
        return path.dirname(path_str);
    }
};



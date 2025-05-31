import type {OptParams} from "../napi-folder/bindings";

export const createOptParams = (
    params: Partial<OptParams>
): OptParams => {
    return {
        path_str: null,
        meta_types: null,
        ordering: null,
        skip_n: null,
        take_n: null,
        is_pretty: null,
        cache_nm: null,
        ...params,
    }
}

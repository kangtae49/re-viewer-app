export const SEP = "\\"

export function getDataset (this: Element) {
    const div = this as HTMLDivElement;
    return div.dataset;
}

export function setDataset (this: Element, obj: Record<string, string | number | boolean | bigint | undefined>) {
    const div = this as HTMLDivElement;
    Object.entries(obj).forEach(([key, value]) => {
        if (typeof value === "boolean" && value == false) {
            return
        }
        if (typeof value === "undefined") {
            return
        }
        div.dataset[key] = String(value);
    });
}





export const isVisibleInViewport = (el: Element, viewEl: Element): boolean => {
    if(!el){
        return true;
    }
    const rect = el.getBoundingClientRect();
    const viewRect = viewEl.getBoundingClientRect();
    return (
        rect.top > viewRect.top &&
        rect.bottom < viewRect.bottom
    );
}

export const shadowHtml = (div: HTMLDivElement, html: string) => {
    const div_shadow = document.createElement("div");
    div_shadow.style.width = "100%";
    div_shadow.style.height = "100%";
    const shadow = div_shadow.attachShadow({ mode: "open" });
    shadow.innerHTML = html;
    div.innerHTML = "";
    div.appendChild(div_shadow);
}

/*
await withLoadingAsync(async () => {
    await asyncFunc();
});
 */
export const withLoadingAsync = async function <T>(fn: () => Promise<T>): Promise<T> {

    try {
        document.documentElement.classList.add("wait-cursor");
        await raf();
        return await fn();
    } finally {
        document.documentElement.classList.remove("wait-cursor");
    }
}

function raf(): Promise<void> {
    return new Promise(resolve => {
        requestAnimationFrame(() => resolve());
    });
}


/*
withLoading(() => {
    syncFunc();
});
 */
export const withLoading  = <T> (fn: () => T): T  => {
    document.documentElement.classList.add("wait-cursor");
    try {
        return fn();
    } finally {
        document.documentElement.classList.remove("wait-cursor");
    }
}

export const isLoading = (): boolean => {
    return document.documentElement.classList.contains("wait-cursor")
}
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






export const getDateFormatter = () => {
    const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
    return new Intl.DateTimeFormat('ko-KR', {
        timeZone: timeZone,
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false
    });
}

export const g_date_formatter = getDateFormatter();

export const toDate = (t: number) => {
    const date = new Date(Number(t) * 1000);
    const formatted = g_date_formatter.format(date);
    const arr = formatted.replace(/\s+/g, "").split(".");
    return arr.slice(0, 3).join("-") + " " + arr.slice(-1)[0]
}



export const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0KB';
    const kb = Math.ceil(bytes / 1024);
    return kb.toLocaleString('en-US') + 'KB';
}

export const isVisibleInViewport = (el: Element, viewEl: Element): boolean => {
    if(!el){
        return true;
    }
    const rect = el.getBoundingClientRect();
    const viewRect = viewEl.getBoundingClientRect();
    return (
        rect.top >= viewRect.top &&
        rect.bottom <= viewRect.bottom
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
        document.documentElement.classList.add("wait-busy");
        document.documentElement.style.pointerEvents = "none";
        await raf();
        return await fn();
    } finally {
        document.documentElement.classList.remove("wait-busy");
        document.documentElement.style.pointerEvents = "";
    }
}

export const isWaitBusy = () => {
    return document.documentElement.classList.contains("wait-busy");
}
export const isWaitResize = () => {
    return document.documentElement.classList.contains("wait-resize");
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
    document.documentElement.style.pointerEvents = "none";
    try {
        return fn();
    } finally {
        document.documentElement.classList.remove("wait-cursor");
        document.documentElement.style.pointerEvents = "";
    }
}

export const isLoading = (): boolean => {
    return document.documentElement.classList.contains("wait-cursor")
}
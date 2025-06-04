export const isVisibleInViewport = (el: HTMLElement, viewEl: HTMLElement): boolean => {
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
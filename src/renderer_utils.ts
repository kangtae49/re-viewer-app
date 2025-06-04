export const isVisibleInViewport = (el: HTMLElement, viewEl: HTMLElement): boolean => {
    const rect = el.getBoundingClientRect();
    const viewRect = viewEl.getBoundingClientRect();
    return (
        rect.top > viewRect.top &&
        rect.bottom < viewRect.bottom
    );
}
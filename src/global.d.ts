declare global {
    interface Element {
        getDataset (this: Element): DOMStringMap;
        setDataset (this: Element, obj: Record<string, string | number | boolean | bigint | undefined>): void;
    }
}

import type {Item} from "../napi-folder/bindings";

declare global {
    interface Element {
        updateItemDataset (this: Element, item: Item, base_path: string): void;
        getDataset (this: Element): DOMStringMap;
        setDataset (this: Element, obj: Record<string, string | number | boolean | bigint | undefined>): void;
    }
}

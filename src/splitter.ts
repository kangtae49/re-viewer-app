import {isWaitBusy, isWaitResize, isVisibleInViewport} from "./renderer_utils"
// type Orientation = "vertical" | "horizontal";
interface SplitterOptions {
    container: string,
    targetA: string,
    targetB: string,
    targetScroll: string | undefined,
    defaultLeft: number,
}

export class Splitter {
    private container: HTMLDivElement;
    private paneA: HTMLDivElement;
    private paneB: HTMLDivElement;
    private scroll: HTMLDivElement;
    private scrollInner: HTMLDivElement;
    private resizer: HTMLDivElement;
    private overlay: HTMLDivElement;

    private targetScroll: HTMLDivElement;

    private isDragging = false;

    private targetA: string;
    private targetB: string;
    private defaultLeft: number;


    constructor(opt: SplitterOptions) {
        this.init(opt);
    }

    private init(opt: SplitterOptions) {
        this.targetA = opt.targetA;
        this.targetB = opt.targetB;
        this.defaultLeft = opt.defaultLeft;
        if (opt.targetScroll) {
            this.targetScroll = document.querySelector(opt.targetScroll);
        }

        this.container = document.createElement("div");
        this.container.classList.add("splitter-container");
        this.paneA = document.createElement("div");
        this.paneA.classList.add("pane-a");
        this.paneB = document.createElement("div");
        this.paneB.classList.add("pane-b");
        this.resizer = document.createElement("div");
        this.resizer.classList.add("resizer");
        this.scroll = document.createElement("div");
        this.scroll.classList.add("scroll");
        this.scrollInner = document.createElement("div");
        this.scrollInner.classList.add("inner");

        this.overlay = document.createElement("div");
        this.overlay.classList.add("overlay");



        this.scroll.appendChild(this.scrollInner);

        const orgA = document.querySelector(this.targetA) as  HTMLDivElement;
        const orgB = document.querySelector(this.targetB) as  HTMLDivElement;
        this.paneA.appendChild(orgA);
        this.paneB.appendChild(orgB);

        this.container.appendChild(this.paneA);
        this.container.appendChild(this.scroll);
        this.container.appendChild(this.resizer);
        this.container.appendChild(this.overlay);
        this.container.appendChild(this.paneB);
        if (!this.targetScroll) {
            this.targetScroll = orgA;
        }

        document.querySelector(opt.container).appendChild(this.container);

        this.resizer.addEventListener("mousedown", this.onMouseDownResizer.bind(this));
        document.addEventListener("mousemove", this.onMouseMoveResizer.bind(this));
        document.addEventListener("mouseup", this.onMouseUpResizer.bind(this));
        this.scroll.addEventListener("scroll", this.onScroll.bind(this));
        this.targetScroll.addEventListener("scroll", this.onScrollTarget.bind(this));
        window.addEventListener("resize", this.resizeWindowEvent.bind(this));
    }


    private onMouseDownResizer() {
        this.isDragging = true;
        this.overlay.classList.add("active");
        document.body.style.cursor = 'ew-resize';
    }

    private onMouseMoveResizer(event: MouseEvent) {
        if (!this.isDragging) return;

        this.resizeLayout(event.clientX);
    }

    private onMouseUpResizer() {
        this.isDragging = false;
        this.overlay.classList.remove("active");
        document.body.style.cursor = '';
    }

    private onScroll() {
        if (this.targetScroll.scrollTop != this.scroll.scrollTop) {
            this.targetScroll.scrollTop = this.scroll.scrollTop;
        }
    }
    private onScrollTarget() {
        if (this.targetScroll.scrollTop != this.scroll.scrollTop) {
            this.scroll.scrollTop = this.targetScroll.scrollTop;
        }
    }


    public resizeLayout (left: number | undefined = undefined) {
        const defaultLeft = this.defaultLeft;
        const resizerWidth = 6;
        const scrollWidth = 18;

        if (left === undefined) {
            left = this.resizer.offsetLeft;
        }
        if (left == 0) {
            left = defaultLeft;
        }

        const minLeft = 0;
        const maxLeft = window.innerWidth; // - 100;
        const resizerLeft = Math.min(Math.max(left, minLeft), maxLeft);
        const contentLeft = resizerLeft + resizerWidth;
        const contentWidth = window.innerWidth - contentLeft;

        this.resizer.style.left = resizerLeft + 'px';
        this.paneB.style.left = contentLeft + 'px';
        this.paneB.style.width = contentWidth + 'px';

        this.scroll.style.left = (resizerLeft - scrollWidth) + 'px';
        this.scroll.style.height = this.targetScroll.clientHeight + 'px';
        this.scroll.style.width = scrollWidth + 'px';
        if (this.targetScroll.clientHeight == this.targetScroll.scrollHeight) {
            this.scroll.style.display = "none";
        } else {
            this.scroll.style.display = "";
        }

        this.scrollInner.style.height = this.targetScroll.scrollHeight + 'px';
        if (this.targetScroll.clientHeight == this.targetScroll.scrollHeight) {
            this.scroll.style.display = "none";
        } else {
            this.scroll.style.display = "";
        }
    }

    private resizeWindowEvent = async () => {
        if (isWaitBusy()) {
            return;
        }
        // const content = this.paneB.querySelector(".content");
        // content.style.display = "none";
        if (!isWaitResize()) {
            // this.isResize = true;
            document.documentElement.classList.add("wait-resize");
            requestAnimationFrame(() => {
                setTimeout(() => {
                    // if (content.style.display == "none") {
                        document.documentElement.classList.remove("wait-resize");
                        // console.log("resizer:" + new Date());
                        // content.style.display = "";
                        this.resizeLayout();
                    // }
                }, 1000);
            });
        }
    }


}


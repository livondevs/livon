export type ComponentDeclaration = (args?: {
  [key: string]: any;
}) => LunasModuleExports;

export type LunasModuleExports = {
  mount: (elm: HTMLElement) => LunasComponentState;
  insert: (elm: HTMLElement, anchor: HTMLElement | null) => LunasComponentState;
  __unmount: () => void;
};

export type LunasComponentState = {
  updatedFlag: boolean;
  valUpdateMap: number;
  blkRenderedMap: number;
  blkUpdateMap: number;
  internalElement: LunasInternalElement;
  currentVarBit: number;
  currentIfBlkBit: number;
  ifBlkRenderers: { [key: string]: () => void };
  isMounted: boolean;
  componentElm: HTMLElement;
  compSymbol: symbol;
  resetDependecies: (() => void)[];
  // componentElmentSetter: (innerHtml: string, topElmTag: string,topElmAttr: {[key: string]: string}) => void
  __lunas_update: () => void;
  __lunas_after_mount: () => void;
  // __lunas_init: () => void;
  // __lunas_destroy: () => void;
  // __lunas_update_component: () => void;
  // __lunas_update_component_end: () => void;
  // __lunas_update_component_start: () => void;
  // __lunas_update_end: () => void;
  // __lunas_update_start: () => void;
  // __lunas_init_component: () => void;
};

type LunasInternalElement = {
  innerHtml: string;
  topElmTag: string;
  topElmAttr: { [key: string]: string };
};

class valueObj<T> {
  dependencies: { [key: symbol]: [LunasComponentState, number] } = {};
  constructor(
    private _v: T,
    componentObj?: LunasComponentState,
    componentSymbol?: symbol,
    symbolIndex: number = 0
  ) {
    if (componentSymbol && componentObj) {
      this.dependencies[componentSymbol] = [componentObj, symbolIndex];
    }
  }

  set v(v: T) {
    if (this._v === v) return;
    this._v = v;
    for (const keys of Object.getOwnPropertySymbols(this.dependencies)) {
      const [componentObj, symbolIndex] = this.dependencies[keys];
      componentObj.valUpdateMap |= symbolIndex;
      if (!componentObj.updatedFlag) {
        Promise.resolve().then(componentObj.__lunas_update.bind(componentObj));
        componentObj.updatedFlag = true;
      }
    }
  }

  get v() {
    return this._v;
  }

  addDependency(componentObj: LunasComponentState, symbolIndex: number) {
    this.dependencies[componentObj.compSymbol] = [componentObj, symbolIndex];
    return {
      removeDependency: () => {
        delete this.dependencies[componentObj.compSymbol];
      },
    };
  }
}

export const $$lunasInitComponent = function (
  this: LunasComponentState,
  args: { [key: string]: any } = {},
  inputs: string[] = []
) {
  this.updatedFlag = false;
  this.valUpdateMap = 0;
  this.blkRenderedMap = 0;
  this.blkUpdateMap = 0;
  this.currentVarBit = 0;
  this.currentIfBlkBit = 0;
  this.isMounted = false;
  this.ifBlkRenderers = {};
  this.compSymbol = Symbol();
  this.resetDependecies = [];

  const genBitOfVariables = function* (this: LunasComponentState) {
    while (true) {
      if (this.currentVarBit === 0) {
        this.currentVarBit = 1;
        yield this.currentVarBit;
      } else {
        this.currentVarBit <<= 1;
        yield this.currentVarBit;
      }
    }
  }.bind(this);

  for (const key of inputs) {
    const arg = args[key];
    if (arg instanceof valueObj) {
      const { removeDependency } = arg.addDependency(
        this,
        genBitOfVariables().next().value
      );
      this.resetDependecies.push(removeDependency);
    } else {
      genBitOfVariables().next();
    }
  }

  const genBitOfIfBlks = function* (this: LunasComponentState) {
    while (true) {
      if (this.currentIfBlkBit === 0) {
        this.currentIfBlkBit = 1;
        yield this.currentIfBlkBit;
      } else {
        this.currentIfBlkBit <<= 1;
        yield this.currentIfBlkBit;
      }
    }
  }.bind(this);

  const componentElementSetter = function (
    this: LunasComponentState,
    innerHtml: string,
    topElmTag: string,
    topElmAttr: { [key: string]: string } = {}
  ) {
    this.internalElement = {
      innerHtml,
      topElmTag,
      topElmAttr,
    };
  }.bind(this);

  const setAfterMount = function (
    this: LunasComponentState,
    afterMount: () => void
  ) {
    this.__lunas_after_mount = afterMount;
  }.bind(this);

  const mount = function (
    this: LunasComponentState,
    elm: HTMLElement
  ): LunasComponentState {
    if (this.isMounted) throw new Error("Component is already mounted");
    elm.innerHTML = `<${this.internalElement.topElmTag} ${Object.keys(
      this.internalElement.topElmAttr
    )
      .map((key) => `${key}="${this.internalElement.topElmAttr[key]}"`)
      .join(" ")}>${this.internalElement.innerHtml}</${
      this.internalElement.topElmTag
    }>`;
    this.componentElm = elm.firstElementChild as HTMLElement;
    this.__lunas_after_mount();
    this.isMounted = true;
    return this;
  }.bind(this);

  const insert = function (
    this: LunasComponentState,
    elm: HTMLElement,
    anchor: HTMLElement | null
  ): LunasComponentState {
    if (this.isMounted) throw new Error("Component is already mounted");
    this.componentElm = createDomElementFromLunasElement(this.internalElement);
    elm.insertBefore(this.componentElm, anchor);
    this.__lunas_after_mount();
    this.isMounted = true;
    return this;
  }.bind(this);

  const __unmount = function (this: LunasComponentState) {
    if (!this.isMounted) throw new Error("Component is not mounted");
    this.componentElm!.remove();
    this.isMounted = false;
    this.resetDependecies.forEach((r) => r());
  }.bind(this);

  const updateComponent = function (
    this: LunasComponentState,
    updateFunc: () => void
  ) {
    this.__lunas_update = (() => {
      if (!this.updatedFlag) return;
      updateFunc.call(this);
      this.updatedFlag = false;
      this.valUpdateMap = 0;
      this.blkUpdateMap = 0;
    }).bind(this);
  }.bind(this);

  const createReactive = function <T>(this: LunasComponentState, v: T) {
    return new valueObj<T>(
      v,
      this,
      this.compSymbol,
      genBitOfVariables().next().value
    );
  }.bind(this);

  const createIfBlock = function (
    this: LunasComponentState,
    name: string,
    lunasElement: () => LunasInternalElement,
    getParentAndRefElement: () => [HTMLElement, HTMLElement | null],
    postRender: () => void
  ) {
    const ifBlkBit = genBitOfIfBlks().next().value;
    this.ifBlkRenderers[name] = (() => {
      const componentElm = createDomElementFromLunasElement(lunasElement());
      const [parentElement, refElement] = getParentAndRefElement();
      parentElement.insertBefore(componentElm, refElement);
      postRender();
      (this.blkRenderedMap |= ifBlkBit), (this.blkUpdateMap |= ifBlkBit);
    }).bind(this);
  }.bind(this);

  const renderIfBlock = function (this: LunasComponentState, name: string) {
    if (!this.ifBlkRenderers[name]) return;
    this.ifBlkRenderers[name]();
  }.bind(this);

  return {
    $$lunasSetComponentElement: componentElementSetter,
    $$lunasUpdateComponent: updateComponent,
    $$lunasAfterMount: setAfterMount,
    $$lunasReactive: createReactive,
    $$lunasCreateIfBlock: createIfBlock,
    $$lunasRenderIfBlock: renderIfBlock,
    $$lunasComponentReturn: {
      mount,
      insert,
      __unmount,
    } as LunasModuleExports,
  };
};

export function $$lunasEscapeHtml(text: any): string {
  const map: { [key: string]: string } = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#039;",
  };

  return String(text).replace(/[&<>"']/g, function (m: string): string {
    return map[m];
  });
}

export function $$lunasGetElmRefs(ids: string[], preserveId: number) {
  return ids.map((id, index) => {
    const e = document.getElementById(id)!;
    (2 ** index) & preserveId && e.removeAttribute("id");
    return e;
  });
}

export function $$lunasAddEvListener(
  elm: HTMLElement,
  evName: string,
  evFunc: EventListener
) {
  elm.addEventListener(evName, evFunc);
}

export function $$lunasReplaceText(content: any, elm: Node) {
  elm.textContent = $$lunasEscapeHtml(content);
}

export function $$lunasReplaceInnerHtml(content: any, elm: HTMLElement) {
  elm.innerHTML = $$lunasEscapeHtml(content);
}

export function $$lunasReplaceAttr(
  key: string,
  content: any,
  elm: HTMLElement
) {
  if (content === undefined && elm.hasAttribute(key)) {
    elm.removeAttribute(key);
    return;
  }
  (elm as any)[key] = String(content);
}

export function $$lunasInsertEmpty(
  parent: HTMLElement,
  anchor: HTMLElement | null
) {
  const empty = document.createTextNode(" ");
  parent.insertBefore(empty, anchor);
  return empty;
}

export function $$lunasInsertContent(
  content: string,
  parent: HTMLElement,
  anchor: HTMLElement | null
) {
  const contentNode = document.createTextNode(content);
  parent.insertBefore(contentNode, anchor);
  return contentNode;
}

export function $$createLunasElement(
  innerHtml: string,
  topElmTag: string,
  topElmAttr: { [key: string]: string }
): LunasInternalElement {
  return {
    innerHtml,
    topElmTag,
    topElmAttr,
  };
}

export const createDomElementFromLunasElement = function (
  lunasElement: LunasInternalElement
): HTMLElement {
  const componentElm = document.createElement(lunasElement.topElmTag);
  Object.keys(lunasElement.topElmAttr).forEach((key) => {
    componentElm.setAttribute(key, lunasElement.topElmAttr[key]);
  });
  componentElm.innerHTML = lunasElement.innerHtml;
  return componentElm;
};

export const $$lunasCreateNonReactive = function <T>(
  this: LunasComponentState,
  v: T
) {
  return new valueObj<T>(v);
};

export function $$lunasShouldRender(blockRendering: boolean, bitValue: number, bitPosition: number): boolean {
  // Get the bit at the specified position (1-based index, so subtract 1)
  const isBitSet = (bitValue >> (bitPosition - 1)) & 1;

  // Compare the block rendering status with the bit status
  return blockRendering !== Boolean(isBitSet);
}

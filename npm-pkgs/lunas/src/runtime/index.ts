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
  internalElement: LunasInternalElement;
  currentVarBit: number;
  ifBlocks: {
    [key: string]: {
      renderer: () => void;
      context: string[];
      forBlk: string | null;
      condition: () => boolean;
      cleanup: (() => void)[];
      childs: string[];
    };
  };
  ifBlockStates: Record<string, boolean>;
  blkUpdateMap: Record<string, boolean>;
  updateComponentFuncs: (() => void)[][];
  isMounted: boolean;
  componentElm: HTMLElement;
  compSymbol: symbol;
  resetDependecies: (() => void)[];
  // componentElmentSetter: (innerHtml: string, topElmTag: string,topElmAttr: {[key: string]: string}) => void
  __lunas_update: (() => void) | undefined;
  __lunas_apply_enhancement: () => void;
  __lunas_after_mount: () => void;
  __lunas_destroy: () => void;
  // __lunas_init: () => void;
  // __lunas_update_component: () => void;
  // __lunas_update_component_end: () => void;
  // __lunas_update_component_start: () => void;
  // __lunas_update_end: () => void;
  // __lunas_update_start: () => void;
  // __lunas_init_component: () => void;
  forBlocks: {
    [key: string]: { cleanUp: (() => void)[]; childs: string[] };
  };

  refMap: RefMap;
};

type LunasInternalElement = {
  innerHtml: string;
  topElmTag: string;
  topElmAttr: { [key: string]: string };
};

type NestedArray<T> = (T | NestedArray<T>)[];

type FragmentFunc = (
  item?: unknown,
  index?: number,
  indices?: number[]
) => Fragment[];

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
    for (const key of Object.getOwnPropertySymbols(this.dependencies)) {
      const [componentObj, symbolIndex] = this.dependencies[key];
      componentObj.valUpdateMap |= symbolIndex;
      if (!componentObj.updatedFlag && componentObj.__lunas_update) {
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
  this.blkUpdateMap = {};
  this.currentVarBit = 0;
  this.isMounted = false;
  this.ifBlocks = {};
  this.ifBlockStates = {};
  this.compSymbol = Symbol();
  this.resetDependecies = [];
  this.refMap = [];
  this.updateComponentFuncs = [[], []];
  this.forBlocks = {};
  this.__lunas_after_mount = () => {};
  this.__lunas_destroy = () => {};

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

  const applyEnhancement = function (
    this: LunasComponentState,
    enhancementFunc: () => void
  ) {
    this.__lunas_apply_enhancement = enhancementFunc;
  }.bind(this);

  const setAfterMount = function (
    this: LunasComponentState,
    afterMount: () => void
  ) {
    this.__lunas_after_mount = afterMount;
  }.bind(this);

  const setAfterUnmount = function (
    this: LunasComponentState,
    afterUnmount: () => void
  ) {
    this.__lunas_destroy = afterUnmount;
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
    this.__lunas_apply_enhancement();
    this.__lunas_after_mount();
    this.isMounted = true;
    _updateComponent(() => {});
    return this;
  }.bind(this);

  const insert = function (
    this: LunasComponentState,
    elm: HTMLElement,
    anchor: HTMLElement | null
  ): LunasComponentState {
    if (this.isMounted) throw new Error("Component is already mounted");
    this.componentElm = _createDomElementFromLunasElement(this.internalElement);
    elm.insertBefore(this.componentElm, anchor);
    this.__lunas_apply_enhancement();
    this.__lunas_after_mount();
    this.isMounted = true;
    return this;
  }.bind(this);

  const __unmount = function (this: LunasComponentState) {
    if (!this.isMounted) throw new Error("Component is not mounted");
    this.componentElm!.remove();
    this.isMounted = false;
    this.resetDependecies.forEach((r) => r());
    this.__lunas_destroy();
  }.bind(this);

  const _updateComponent = function (
    this: LunasComponentState,
    updateFunc: () => void
  ) {
    this.__lunas_update = (() => {
      if (!this.updatedFlag) return;
      this.updateComponentFuncs[0].forEach((f) => f());
      this.updateComponentFuncs[1].forEach((f) => f());
      updateFunc.call(this);
      this.updatedFlag = false;
      this.valUpdateMap = 0;
      this.blkUpdateMap = {};
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
    ifBlocks: [
      name: string | (() => string),
      lunasElement: () => LunasInternalElement,
      condition: () => boolean,
      postRender: () => void,
      ifCtx: string[],
      forCtx: string[],
      depBit: number,
      mapInfo: [mapOffset: number | number[], mapLength: number],
      refIdx: [
        parentElementIndex: number | number[],
        refElementIndex?: number | number[]
      ],
      fragment?: Fragment[]
    ][],
    indices?: number[]
  ) {
    for (const [
      getName,
      lunasElement,
      condition,
      postRender,
      ifCtxUnderFor,
      forCtx,
      depBit,
      [mapOffset, mapLength],
      [parentElementIndex, refElementIndex],
      fragments,
    ] of ifBlocks) {
      const name = typeof getName === "function" ? getName() : getName;
      this.ifBlocks[name] = {
        renderer: ((
          mapOffset: number | number[],
          _mapLength: number | number[]
        ) => {
          const componentElm = _createDomElementFromLunasElement(
            lunasElement()
          );
          const parentElement = getNestedArrayValue(
            this.refMap,
            parentElementIndex
          ) as HTMLElement;
          const refElement = getNestedArrayValue(this.refMap, refElementIndex);
          parentElement!.insertBefore(componentElm, refElement ?? null);
          setNestedArrayValue(this.refMap, mapOffset, componentElm);
          postRender();
          if (fragments) {
            createFragments(
              fragments,
              [...ifCtxUnderFor, name],
              forCtx[forCtx.length - 1]
            );
          }
          this.ifBlockStates[name] = true;
          this.blkUpdateMap[name] = true;
          Object.values(this.ifBlocks).forEach((blk) => {
            if (blk.context.includes(name)) {
              blk.condition() && blk.renderer();
            }
          });
        }).bind(this, mapOffset, mapLength),
        context: ifCtxUnderFor.map((ctx) =>
          indices ? `${ctx}-${indices}` : ctx
        ),
        condition,
        forBlk: forCtx.length ? forCtx[forCtx.length - 1] : null,
        cleanup: [],
        childs: [],
      };

      ifCtxUnderFor.forEach((ctx) => {
        const parentBlockName = indices ? `${ctx}-${indices}` : ctx;
        this.ifBlocks[parentBlockName].childs.push(name);
      });

      const updateFunc = (() => {
        if (this.valUpdateMap & depBit) {
          const shouldRender = condition();
          const rendered = !!this.ifBlockStates[name];
          const parentRendered = ifCtxUnderFor.every(
            (ctx) => this.ifBlockStates[indices ? `${ctx}-${indices}` : ctx]
          );
          if (shouldRender && !rendered && parentRendered) {
            this.ifBlocks[name].renderer();
          } else if (!shouldRender && rendered) {
            const ifBlkElm = getNestedArrayValue(
              this.refMap,
              mapOffset
            ) as HTMLElement;
            ifBlkElm.remove();
            if (typeof mapOffset === "number") {
              this.refMap.fill(undefined, mapOffset, mapOffset + mapLength);
            } else {
              for (let i = 0; i < mapLength; i++) {
                const copiedMapOffset = [...mapOffset];
                copiedMapOffset[0] += i;
                setNestedArrayValue(this.refMap, copiedMapOffset, undefined);
              }
            }

            delete this.ifBlockStates[name];

            [name, ...this.ifBlocks[name].childs].forEach((child) => {
              if (this.ifBlocks[child]) {
                this.ifBlocks[child].cleanup.forEach((f) => f());
                this.ifBlocks[child].cleanup = [];
              }
            });
          }
        }
      }).bind(this);

      this.updateComponentFuncs[0].push(updateFunc);

      if (ifCtxUnderFor.length === 0) {
        condition() && this.ifBlocks[name].renderer();
      } else {
        const parentBlockName = ifCtxUnderFor[ifCtxUnderFor.length - 1];
        if (this.ifBlockStates[parentBlockName] && condition()) {
          this.ifBlocks[name].renderer();
        }
      }
    }
    this.blkUpdateMap = {};
  }.bind(this);

  const renderIfBlock = function (this: LunasComponentState, name: string) {
    if (!this.ifBlocks[name]) return;
    this.ifBlocks[name].renderer();
  }.bind(this);

  const getElmRefs = function (
    this: LunasComponentState,
    ids: string[],
    preserveId: number,
    refLocation: number | number[] = 0
  ): void {
    ids.forEach(
      function (this: LunasComponentState, id: string, index: number) {
        const e = document.getElementById(id)!;
        (2 ** index) & preserveId && e.removeAttribute("id");
        const newRefLocation = addNumberToArrayInitial(refLocation, index);
        setNestedArrayValue(this.refMap, newRefLocation, e);
      }.bind(this)
    );
  }.bind(this);

  const addEvListener = function (
    this: LunasComponentState,
    args: [number | number[], string, EventListener][]
  ) {
    for (const [elmIdx, evName, evFunc] of args) {
      const target = getNestedArrayValue(this.refMap, elmIdx) as HTMLElement;
      target.addEventListener(evName, evFunc);
    }
  }.bind(this);

  const createForBlock = function (
    this: LunasComponentState,
    forBlocksConfig: [
      forBlockId: string,
      renderItem: (
        item: unknown,
        index: number,
        indices: number[]
      ) => LunasInternalElement,
      getDataArray: () => unknown[],
      afterRenderHook: (
        item: unknown,
        index: number,
        indices: number[]
      ) => void,
      ifCtxUnderFor: string[],
      forCtx: string[],
      updateFlag: number,
      parentIndices: number[],
      mapInfo: [mapOffset: number, mapLength: number],
      refIdx: [
        parentElementIndex: number | number[],
        refElementIndex?: number | number[]
      ],
      fragment?: FragmentFunc
    ][]
  ): void {
    for (const config of forBlocksConfig) {
      const [
        forBlockId,
        renderItem,
        getDataArray,
        afterRenderHook,
        ifCtxUnderFor,
        forCtx,
        updateFlag,
        parentIndices,
        [mapOffset, mapLength],
        [parentElementIndex, refElementIndex],
        fragmentFunc,
      ] = config;

      this.forBlocks[forBlockId] = { cleanUp: [], childs: [] };
      forCtx.forEach((ctx) => {
        this.forBlocks[ctx].childs.push(forBlockId);
      });

      const renderForBlock = ((items: unknown[]) => {
        const containerElm = getNestedArrayValue(
          this.refMap,
          parentElementIndex
        ) as HTMLElement;

        const insertionPointElm = getNestedArrayValue(
          this.refMap,
          refElementIndex
        ) as HTMLElement;
        items.forEach((item, index) => {
          const fullIndices = [...parentIndices, index];
          const lunasElm = renderItem(item, index, fullIndices);
          const domElm = _createDomElementFromLunasElement(lunasElm);
          setNestedArrayValue(this.refMap, [mapOffset, ...fullIndices], domElm);
          containerElm.insertBefore(domElm, insertionPointElm);
          afterRenderHook?.(item, index, fullIndices);
          if (fragmentFunc) {
            const fragments = fragmentFunc(item, index, fullIndices);
            createFragments(fragments, ifCtxUnderFor, forBlockId);
          }
        });
      }).bind(this);
      renderForBlock(getDataArray());

      let oldItems: null | unknown[] = null;

      this.updateComponentFuncs[0].push(
        (() => {
          if (this.valUpdateMap & updateFlag) {
            const newItems = getDataArray();
            // FIXME: Improve the logic to handle updates properly
            if (oldItems === null || diffDetected(oldItems, newItems)) {
              const refArr = this.refMap[mapOffset] as RefMapItem[];
              // Iterate in reverse order to prevent index shift issues when removing elements
              for (let i = refArr.length - 1; i >= 0; i--) {
                const item = refArr[i];
                if (item instanceof HTMLElement) {
                  item.remove();
                  refArr.splice(i, 1);
                }
              }
              if (this.forBlocks[forBlockId]) {
                const { cleanUp, childs } = this.forBlocks[forBlockId];
                cleanUp.forEach((f) => f());
                Array.from(childs).forEach((child) => {
                  if (this.forBlocks[child]) {
                    this.forBlocks[child]!.cleanUp.forEach((f) => f());
                    this.forBlocks[child].cleanUp = [];
                  }
                });
                this.forBlocks[forBlockId].cleanUp = [];
              }
              renderForBlock(newItems);
            }
            oldItems = newItems;
          }
        }).bind(this)
      );
    }
  }.bind(this);

  const insertTextNodes = function (
    this: LunasComponentState,
    args: [
      amount: number,
      parent: number | number[],
      anchor?: number | number[],
      text?: string
    ][],
    _assignmentLocation: number[] | number = 0
  ) {
    let assignmentLocation =
      typeof _assignmentLocation === "number"
        ? [_assignmentLocation]
        : _assignmentLocation;
    for (const [amount, parentIdx, anchorIdx, text] of args) {
      for (let i = 0; i < amount; i++) {
        const txtNode = document.createTextNode(text ?? " ");
        const parentElm = getNestedArrayValue(
          this.refMap,
          parentIdx
        ) as HTMLElement;
        const anchorElm = getNestedArrayValue(
          this.refMap,
          anchorIdx
        ) as HTMLElement;
        parentElm.insertBefore(txtNode, anchorElm);
        setNestedArrayValue(this.refMap, assignmentLocation, txtNode);
        assignmentLocation[0]++;
      }
    }
  }.bind(this);

  const createFragments = function (
    this: LunasComponentState,
    fragments: Fragment[],
    ifCtx?: string[],
    latestForName?: string
  ) {
    for (const [
      [textContent, attributeName],
      _nodeIdx,
      depBit,
      fragmentType,
    ] of fragments) {
      const nodeIdx = typeof _nodeIdx === "number" ? [_nodeIdx] : _nodeIdx;
      const fragmentUpdateFunc = (() => {
        if (ifCtx?.length) {
          const blockRendered = ifCtx.every(
            (ctxName) => this.ifBlockStates[ctxName]
          );
          const blockAlreadyUpdated = ifCtx.every(
            (ctxName) => this.blkUpdateMap[ctxName]
          );
          if (!blockRendered || blockAlreadyUpdated) {
            return;
          }
        }
        const valueUpdated = (this.valUpdateMap & depBit) !== 0;
        if (!valueUpdated) {
          return;
        }
        const target = getNestedArrayValue(this.refMap, nodeIdx) as Node;
        if (fragmentType === FragmentType.ATTRIBUTE) {
          $$lunasReplaceAttr(
            attributeName!,
            textContent(),
            target as HTMLElement
          );
        } else {
          $$lunasReplaceText(textContent(), target);
        }
      }).bind(this);
      if (fragmentType === FragmentType.ATTRIBUTE) {
        // Because the determination of the arribute types depends on dynamic values,
        // it is necessary to update the attributes after the initial rendering
        const target = getNestedArrayValue(this.refMap, nodeIdx) as Node;
        $$lunasReplaceAttr(
          attributeName!,
          textContent(),
          target as HTMLElement
        );
      }
      this.updateComponentFuncs[1].push(fragmentUpdateFunc);
      if (latestForName) {
        const cleanUpFunc = (() => {
          const idx = this.updateComponentFuncs[1].indexOf(fragmentUpdateFunc);
          this.updateComponentFuncs[1].splice(idx, 1);
        }).bind(this);
        this.forBlocks[latestForName]!.cleanUp.push(cleanUpFunc);
      }
    }
  }.bind(this);

  const lunasInsertComponent = function (
    this: LunasComponentState,
    componentExport: LunasModuleExports,
    parentIdx: number | number[],
    anchorIdx: number | number[] | null,
    refIdx: number | number[],
    latestCtx: string | null,
    indices: number[] | null
  ) {
    const parentElement = getNestedArrayValue(
      this.refMap,
      parentIdx
    ) as HTMLElement;
    const anchorElement = getNestedArrayValue(
      this.refMap,
      anchorIdx
    ) as HTMLElement;
    const { componentElm } = componentExport.insert(
      parentElement,
      anchorElement
    );
    setNestedArrayValue(this.refMap, refIdx, componentElm);
    if (latestCtx) {
      const blockName = indices ? `${latestCtx}-${indices}` : latestCtx;
      if (this.forBlocks[blockName]) {
        this.forBlocks[blockName].cleanUp.push(() => {
          componentExport.__unmount();
        });
      } else if (this.ifBlocks[blockName]) {
        this.ifBlocks[blockName].cleanup.push(() => {
          componentExport.__unmount();
        });
      }
    }
  }.bind(this);

  const lunasMountComponent = function (
    this: LunasComponentState,
    componentExport: LunasModuleExports,
    parentIdx: number,
    refIdx: number
  ) {
    this.refMap[refIdx] = componentExport.mount(
      this.refMap[parentIdx] as HTMLElement
    ).componentElm;
  }.bind(this);

  return {
    $$lunasSetComponentElement: componentElementSetter,
    $$lunasApplyEnhancement: applyEnhancement,
    $$lunasAfterMount: setAfterMount,
    $$lunasAfterUnmount: setAfterUnmount,
    $$lunasReactive: createReactive,
    $$lunasCreateIfBlock: createIfBlock,
    $$lunasCreateForBlock: createForBlock,
    $$lunasRenderIfBlock: renderIfBlock,
    $$lunasGetElmRefs: getElmRefs,
    $$lunasInsertTextNodes: insertTextNodes,
    $$lunasAddEvListener: addEvListener,
    $$lunasCreateFragments: createFragments,
    $$lunasInsertComponent: lunasInsertComponent,
    $$lunasMountComponent: lunasMountComponent,
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

export function $$lunasReplaceText(content: any, elm: Node) {
  elm.textContent = $$lunasEscapeHtml(content);
}

// export function $$lunasReplaceInnerHtml(content: any, elm: HTMLElement) {
//   elm.innerHTML = $$lunasEscapeHtml(content);
// }

export function $$lunasReplaceAttr(
  key: string,
  content: any,
  elm: HTMLElement
) {
  if (typeof content === "boolean") {
    if (content) {
      elm.setAttribute(key, "");
    } else if (elm.hasAttribute(key)) {
      elm.removeAttribute(key);
    }
    return;
  }
  if (content === undefined && elm.hasAttribute(key)) {
    elm.removeAttribute(key);
    return;
  }
  (elm as any)[key] = String(content);
}

export function $$createLunasElement(
  innerHtml: string,
  topElmTag: string,
  topElmAttr: { [key: string]: string } = {}
): LunasInternalElement {
  return {
    innerHtml,
    topElmTag,
    topElmAttr,
  };
}

const _createDomElementFromLunasElement = function (
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

const _shouldRender = (
  blockRendering: boolean,
  bitValue: number,
  bitPosition: number
): boolean => {
  // Get the bit at the specified position (1-based index, so subtract 1)
  const isBitSet = (bitValue & bitPosition) > 0;

  // Compare the block rendering status with the bit status
  return blockRendering !== Boolean(isBitSet);
};

type Fragment = [
  content: [textContent: () => string, attributeName?: string],
  nodeIdx: number[] | number,
  depBit: number,
  fragmentType: FragmentType
];

type RefMapItem = Node | undefined | RefMapItem[];
type RefMap = RefMapItem[];

enum FragmentType {
  ATTRIBUTE = 0,
  TEXT = 1,
  ELEMENT = 2,
}

function diffDetected<T>(oldArray: T[], newArray: T[]): boolean {
  // return (
  //   oldArray.length !== newArray.length ||
  //   oldArray.some((v, i) => v !== newArray[i])
  // );
  // FIXME: This is a temporary implementation
  return true;
}

function setNestedArrayValue<T>(
  arr: NestedArray<T>,
  location: number | number[],
  value: T
): void {
  const path = numberOrNumberArrayToNumberArray(location);
  let current: any = arr;
  for (let i = 0; i < path.length - 1; i++) {
    const key = path[i];
    if (current[key] === undefined) {
      current[key] = [];
    }
    current = current[key];
  }
  current[path[path.length - 1]] = value;
}

function getNestedArrayValue<T>(
  arr: NestedArray<T>,
  location: number | number[] | null | undefined
): T | null {
  if (location == null) return null;
  const path = numberOrNumberArrayToNumberArray(location);
  let current: any = arr;
  for (const key of path) {
    if (!Array.isArray(current) || current[key] == null) {
      return null;
    }
    current = current[key];
  }
  return current as T;
}

function numberOrNumberArrayToNumberArray(
  location: number | number[]
): number[] {
  return typeof location === "number" ? [location] : location;
}

function addNumberToArrayInitial(
  arr: number[] | number,
  num: number
): number[] {
  if (typeof arr === "number") {
    return [arr + num];
  } else {
    const copy = [...arr];
    copy[0] += num;
    return copy;
  }
}

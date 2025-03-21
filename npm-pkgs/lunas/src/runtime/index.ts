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
  currentForBlkBit: number;
  ifBlocks: {
    [key: string]: {
      renderer: () => void;
      context: string[];
      condition: () => boolean;
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
  cleanUps: { [key: string]: (() => void)[] };

  refMap: RefMap;
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
    for (const key of Object.getOwnPropertySymbols(this.dependencies)) {
      const [componentObj, symbolIndex] = this.dependencies[key];
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
  this.blkUpdateMap = {};
  this.currentVarBit = 0;
  this.currentForBlkBit = 0; // TODO: Delete this
  this.isMounted = false;
  this.ifBlocks = {};
  this.ifBlockStates = {};
  this.compSymbol = Symbol();
  this.resetDependecies = [];
  this.refMap = [];
  this.updateComponentFuncs = [[], []];
  this.cleanUps = {};

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
  const genBitOfForBlock = function* (this: LunasComponentState) {
    while (true) {
      if (this.currentForBlkBit === 0) {
        this.currentForBlkBit = 1;
        yield this.currentForBlkBit;
      } else {
        this.currentForBlkBit <<= 1;
        yield this.currentForBlkBit;
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
      name: string,
      lunasElement: () => LunasInternalElement,
      condition: () => boolean,
      postRender: () => void,
      additionalCtx: string[],
      forCtx: string[],
      depBit: number,
      mapInfo: [mapOffset: number | number[], mapLength: number],
      refIdx: [parentElementIndex: number, refElementIndex?: number],
      fragment?: Fragment[]
    ][]
  ) {
    for (const [
      name,
      lunasElement,
      condition,
      postRender,
      ifCtx,
      forCtx,
      depBit,
      [mapOffset, mapLength],
      [parentElementIndex, refElementIndex],
      fragments,
    ] of ifBlocks) {
      this.ifBlocks[name] = {
        renderer: ((
          mapOffset: number | number[],
          _mapLength: number | number[]
        ) => {
          const componentElm = _createDomElementFromLunasElement(
            lunasElement()
          );
          const [parentLocationArray, parentOffset] = getNestedArrayAndItem(
            parentElementIndex,
            this.refMap
          );
          const parentElement = parentLocationArray[
            parentOffset
          ] as HTMLElement;
          const refElement = (() => {
            if (refElementIndex == undefined) {
              return null;
            }
            const [refLocationArray, refOffset] = getNestedArrayAndItem(
              refElementIndex,
              this.refMap
            );
            return refLocationArray[refOffset] as HTMLElement;
          })();
          parentElement!.insertBefore(componentElm, refElement);
          const [referenceMapping, offset] = obtainNestedArrayPositionAndReset(
            mapOffset,
            this.refMap
          );
          referenceMapping[offset] = componentElm;
          postRender();
          this.ifBlockStates[name] = true;
          this.blkUpdateMap[name] = true;
          Object.values(this.ifBlocks).forEach((blk) => {
            if (blk.context.includes(name)) {
              blk.condition() && blk.renderer();
            }
          });
        }).bind(this, mapOffset, mapLength),
        context: ifCtx,
        condition,
      };

      const updateFunc = (() => {
        if (this.valUpdateMap & depBit) {
          const shouldRender = condition();
          const rendered = !!this.ifBlockStates[name];
          if (shouldRender && !rendered) {
            this.ifBlocks[name].renderer();
          } else if (!shouldRender && rendered) {
            const [locationArray, offset] = getNestedArrayAndItem(
              mapOffset,
              this.refMap
            );
            const ifBlkElm = locationArray[offset] as HTMLElement;
            ifBlkElm.remove();

            if (typeof mapOffset === "number") {
              this.refMap.fill(undefined, mapOffset, mapOffset + mapLength);
            } else {
              for (let i = 0; i < mapLength; i++) {
                const copiedMapOffset = [...mapOffset];
                copiedMapOffset[0] += i;
                const [locationArray, offset] = getNestedArrayAndItem(
                  copiedMapOffset,
                  this.refMap
                );
                locationArray[offset] = undefined;
              }
            }

            delete this.ifBlockStates[name];
          }
        }
      }).bind(this);

      this.updateComponentFuncs[0].push(updateFunc);

      if (forCtx.length) {
        const latest = forCtx[forCtx.length - 1];
        (this.cleanUps[latest] ??= []).push(() => {
          const idx = this.updateComponentFuncs[0].indexOf(updateFunc);
          this.updateComponentFuncs[0].splice(idx, 1);
        });
      }

      if (fragments?.length) {
        const newCtx = [...ifCtx, name].filter((item) =>
          Object.keys(this.ifBlocks).includes(item)
        );
        createFragments(fragments, newCtx);
      }

      if (ifCtx.length === 0) {
        condition() && this.ifBlocks[name].renderer();
      } else {
        const parentBlockName = ifCtx[ifCtx.length - 1];
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
    offsetLocation: number | number[] = 0
  ): void {
    // TODO: 以下の関数をutilに移す
    ids.forEach(
      function (this: LunasComponentState, id: string, index: number) {
        const [referenceMapping, offset] = obtainNestedArrayPositionAndReset(
          offsetLocation,
          this.refMap,
          index
        );
        const e = document.getElementById(id)!;
        (2 ** index) & preserveId && e.removeAttribute("id");
        referenceMapping[offset] = e;
      }.bind(this)
    );
  }.bind(this);

  const addEvListener = function (
    this: LunasComponentState,
    args: [number | number[], string, EventListener][]
  ) {
    for (const [elmIdx, evName, evFunc] of args) {
      const [locationArray, offset] = getNestedArrayAndItem(
        elmIdx,
        this.refMap
      );
      const target = locationArray[offset] as HTMLElement;
      target.addEventListener(evName, evFunc);
    }
  }.bind(this);

  const createForBlock = function (
    this: LunasComponentState,
    forBlocksConfig: [
      forBlockId: string,
      renderItem: (item: unknown, index: number) => LunasInternalElement,
      getDataArray: () => unknown[],
      afterRenderHook: (item: unknown, index: number) => void,
      updateFlag: number,
      mapInfo: [mapOffset: number, mapLength: number],
      refIdx: [parentElementIndex: number, refElementIndex?: number],
      context: any,
      containerRef: string,
      insertionPointRef: string | null,
      additionalParams: { [key: string]: any }
    ][]
  ): void {
    for (const config of forBlocksConfig) {
      const [
        forBlockId,
        renderItem,
        getDataArray,
        afterRenderHook,
        updateFlag,
        [mapOffset, mapLength],
        [parentElementIndex, refElementIndex],
        // TODO: Decide whether to use the following or delete it
        // context,
        // updateFlag,
        // containerRef,
        // insertionPointRef,
        // additionalParams,
      ] = config;

      // 初回レンダリング
      let items = getDataArray();
      const uniqueBit = genBitOfForBlock().next().value;

      const containerElm = this.refMap[parentElementIndex] as HTMLElement;

      const insertionPointElm =
        refElementIndex == undefined
          ? null
          : (this.refMap[refElementIndex] as HTMLElement);

      this.refMap[mapOffset] = [] as HTMLElement[];
      items.forEach((item, index) => {
        const lunasElm = renderItem(item, index);
        const domElm = _createDomElementFromLunasElement(lunasElm);

        (this.refMap[mapOffset]! as HTMLElement[]).push(domElm);
        containerElm.insertBefore(domElm, insertionPointElm);
        afterRenderHook?.(item, index);
      });

      this.updateComponentFuncs[0].push(
        (() => {
          if (this.valUpdateMap & updateFlag) {
            const newItems = getDataArray();
            // FIXME: Improve the logic to handle updates properly
            if (diffDetected(items, newItems)) {
              updateForBlock(
                forBlockId,
                newItems,
                renderItem,
                afterRenderHook,
                containerElm,
                insertionPointElm,
                mapOffset,
                this.refMap,
                uniqueBit
              );
              if (this.cleanUps[forBlockId]) {
                this.cleanUps[forBlockId].forEach((f) => f());
                delete this.cleanUps[forBlockId];
              }
              newItems.forEach((item, index) => {
                const lunasElm = renderItem(item, index);
                const domElm = _createDomElementFromLunasElement(lunasElm);
                (this.refMap[mapOffset]! as HTMLElement[]).push(domElm);
                containerElm.insertBefore(domElm, insertionPointElm);
                afterRenderHook && afterRenderHook(item, index);
              });
            }
            items = newItems;
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
    _offset: number = 0
  ) {
    let offset = _offset;
    for (const [amount, parentIdx, anchorIdx, text] of args) {
      for (let i = 0; i < amount; i++) {
        const empty = document.createTextNode(text ?? " ");
        const parent = (() => {
          const [locationArray, offset] = getNestedArrayAndItem(
            parentIdx,
            this.refMap
          );
          return locationArray[offset] as HTMLElement;
        })();
        const anchor = (() => {
          if (anchorIdx == undefined) return null;
          const [locationArray, offset] = getNestedArrayAndItem(
            anchorIdx,
            this.refMap
          );
          return locationArray[offset] as HTMLElement;
        })();
        parent.insertBefore(empty, anchor);
        this.refMap[offset + i] = empty;
      }
      offset += amount;
    }
  }.bind(this);

  const createFragments = function (
    this: LunasComponentState,
    fragments: Fragment[],
    ifCtx?: string[]
  ) {
    for (const [
      [textContent, attributeName],
      nodeIdx,
      depBit,
      fragmentType,
    ] of fragments) {
      this.updateComponentFuncs[1].push(
        (() => {
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
          const target = this.refMap[nodeIdx] as HTMLElement;
          if (fragmentType === FragmentType.ATTRIBUTE) {
            $$lunasReplaceAttr(
              attributeName!,
              textContent(),
              target as HTMLElement
            );
          } else {
            $$lunasReplaceText(textContent(), target);
          }
        }).bind(this)
      );
    }
  }.bind(this);

  const lunasInsertComopnent = function (
    this: LunasComponentState,
    componentExport: LunasModuleExports,
    parentIdx: number | number[],
    anchorIdx: number | number[] | null,
    refIdx: number | number[]
  ) {
    const [parentLocationArray, parentOffset] =
      obtainNestedArrayPositionAndReset(parentIdx, this.refMap);
    const parentElement = parentLocationArray[parentOffset] as HTMLElement;
    const anchorElement = (() => {
      if (anchorIdx == undefined) {
        return null;
      }
      const [anchorLocationArray, anchorOffset] =
        obtainNestedArrayPositionAndReset(anchorIdx, this.refMap);
      return anchorLocationArray[anchorOffset] as HTMLElement;
    })();
    const [referenceMapping, offset] = obtainNestedArrayPositionAndReset(
      refIdx,
      this.refMap
    );
    referenceMapping[offset] = componentExport.insert(
      parentElement,
      anchorElement
    ).componentElm;
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
    $$lunasAfterMount: setAfterMount,
    $$lunasReactive: createReactive,
    $$lunasCreateIfBlock: createIfBlock,
    $$lunasCreateForBlock: createForBlock,
    $$lunasRenderIfBlock: renderIfBlock,
    $$lunasGetElmRefs: getElmRefs,
    $$lunasInsertTextNodes: insertTextNodes,
    $$lunasAddEvListener: addEvListener,
    $$lunasCreateFragments: createFragments,
    $$lunasInsertComponent: lunasInsertComopnent,
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
  nodeIdx: number,
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

// TODO: The initial rendering and the update process have almost the same logic, so it should be unified.
function updateForBlock(
  _forBlockId: string,
  newItems: unknown[],
  renderItem: (item: unknown, index: number) => LunasInternalElement,
  afterRenderHook: (item: unknown, index: number) => void,
  containerElm: HTMLElement,
  insertionPointElm: HTMLElement | null,
  mapOffset: number,
  refMap: RefMap,
  _forBlkBit: number
): void {
  if (!containerElm) return;
  const refArr = refMap[mapOffset] as RefMapItem[];
  // Iterate in reverse order to prevent index shift issues when removing elements
  for (let i = refArr.length - 1; i >= 0; i--) {
    const item = refArr[i];
    if (item instanceof HTMLElement) {
      item.remove();
      refArr.splice(i, 1);
    }
  }
}

// TODO: Currently, getNestedArrayAndItem and its caller are combined for processing,
// but consider whether it is better to handle everything within a function.
function splitArrayAndOffset(
  location: number[] | number
): [arr: number[], offset: number] {
  if (typeof location === "number") {
    return [[], location];
  }
  return [location.slice(0, -1), location[location.length - 1]];
}

function getNestedArrayAndItem<T>(
  location: number[] | number,
  array: T[]
): [arr: T[], offset: number] {
  const [arr, finalLoc] = splitArrayAndOffset(location);
  return [
    arr.reduce((acc, idx) => {
      return acc[idx] as T[];
    }, array),
    finalLoc,
  ];
}

type NestedArray<T> = (T | NestedArray<T>)[];

function obtainNestedArrayPositionAndReset<T>(
  location: number[] | number,
  array: NestedArray<T>,
  offset: number = 0
): [arr: NestedArray<T>, offset: number] {
  // Update return type to NestedArray<T>
  let [arr, finalLoc] = splitArrayAndOffset(location);
  if (arr.length === 0 && offset !== 0) {
    finalLoc += offset;
  } else if (arr.length !== 0 && offset !== 0) {
    arr[0] += offset;
  }
  const finalArray = arr.reduce((acc, idx) => {
    if (!acc[idx]) {
      acc[idx] = [] as NestedArray<T>; // Update type to NestedArray<T>
    }
    return acc[idx] as NestedArray<T>;
  }, array);
  return [finalArray as NestedArray<T>, finalLoc];
}

export type ComponentDeclaration = (args?: {
  [key: string]: any;
}) => LivonModuleExports;

export type LivonModuleExports = {
  mount: (elm: HTMLElement) => LivonComponentState;
  insert: (elm: HTMLElement, anchor: HTMLElement | null) => LivonComponentState;
  __unmount: () => void;
};

export type LivonComponentState = {
  updatedFlag: boolean;
  valUpdateMap: number[];
  internalElement: LivonInternalElement;
  currentVarBitGen: Generator<number[]>;
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
  __livon_update: (() => void) | undefined;
  __livon_apply_enhancement: () => void;
  __livon_after_mount: () => void;
  __livon_destroy: () => void;
  // __livon_init: () => void;
  // __livon_update_component: () => void;
  // __livon_update_component_end: () => void;
  // __livon_update_component_start: () => void;
  // __livon_update_end: () => void;
  // __livon_update_start: () => void;
  // __livon_init_component: () => void;
  forBlocks: {
    [key: string]: { cleanUp: (() => void)[]; childs: string[] };
  };

  refMap: RefMap;
};

type LivonInternalElement = {
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
  private _v: T;
  private proxy: T;
  // Dependencies map: key is a symbol, value is a tuple of [LivonComponentState, number[]]
  dependencies: { [key: symbol]: [LivonComponentState, number[]] } = {};

  constructor(
    initialValue: T,
    componentObj?: LivonComponentState,
    componentSymbol?: symbol,
    symbolIndex: number[] = [0]
  ) {
    this._v = initialValue;

    if (componentSymbol && componentObj) {
      this.dependencies[componentSymbol] = [componentObj, symbolIndex];
    }

    // If the initial value is an object (and not null), wrap it with a Proxy
    if (typeof initialValue === "object" && initialValue !== null) {
      this.proxy = this.createProxy(initialValue);
    } else {
      this.proxy = initialValue;
    }
  }

  set v(v: T) {
    if (this._v === v) return;
    this._v = v;
    // If the new value is an object, wrap it with a Proxy
    if (typeof v === "object" && v !== null) {
      this.proxy = this.createProxy(v);
    } else {
      this.proxy = v;
    }
    this.triggerUpdate();
  }

  get v() {
    return this.proxy;
  }

  // Triggers an update for all dependencies
  private triggerUpdate() {
    for (const key of Object.getOwnPropertySymbols(this.dependencies)) {
      const [componentObj, symbolIndex] = this.dependencies[key];
      bitOrAssign(componentObj.valUpdateMap, symbolIndex);
      if (!componentObj.updatedFlag && componentObj.__livon_update) {
        Promise.resolve().then(componentObj.__livon_update.bind(componentObj));
        componentObj.updatedFlag = true;
      }
    }
  }

  // Creates a Proxy recursively to detect changes in nested objects and arrays
  private createProxy(target: any): any {
    const self = this;
    // If target is not an object or is null, return it directly
    if (typeof target !== "object" || target === null) {
      return target;
    }
    return new Proxy(target, {
      get(target, prop, receiver) {
        const value = Reflect.get(target, prop, receiver);
        // Wrap array mutation methods to trigger update
        if (
          Array.isArray(target) &&
          typeof value === "function" &&
          [
            "push",
            "pop",
            "shift",
            "unshift",
            "splice",
            "sort",
            "reverse",
          ].includes(prop.toString())
        ) {
          return function (...args: any[]) {
            const result = value.apply(target, args);
            self.triggerUpdate();
            return result;
          };
        }
        // If the value is an object, return a Proxy for it (recursive wrapping)
        if (typeof value === "object" && value !== null) {
          return self.createProxy(value);
        }
        return value;
      },
      set(target, prop, value, receiver) {
        const oldVal = target[prop as keyof typeof target];
        if (oldVal === value) return true;
        // If the new value is an object, wrap it with a Proxy before setting it
        const newValue =
          typeof value === "object" && value !== null
            ? self.createProxy(value)
            : value;
        const result = Reflect.set(target, prop, newValue, receiver);
        self.triggerUpdate();
        return result;
      },
    });
  }

  // Adds a dependency and returns a removal function
  addDependency(componentObj: LivonComponentState, symbolIndex: number[]) {
    this.dependencies[componentObj.compSymbol] = [componentObj, symbolIndex];
    return {
      removeDependency: () => {
        delete this.dependencies[componentObj.compSymbol];
      },
    };
  }
}

export const $$livonInitComponent = function (
  this: LivonComponentState,
  args: { [key: string]: any } = {},
  inputs: string[] = []
) {
  this.updatedFlag = false;
  this.valUpdateMap = [0];
  this.blkUpdateMap = {};
  this.currentVarBitGen = bitArrayGenerator();
  this.isMounted = false;
  this.ifBlocks = {};
  this.ifBlockStates = {};
  this.compSymbol = Symbol();
  this.resetDependecies = [];
  this.refMap = [];
  this.updateComponentFuncs = [[], []];
  this.forBlocks = {};
  this.__livon_after_mount = () => {};
  this.__livon_destroy = () => {};

  for (const key of inputs) {
    const arg = args[key];
    if (arg instanceof valueObj) {
      const { removeDependency } = arg.addDependency(
        this,
        this.currentVarBitGen.next().value
      );
      this.resetDependecies.push(removeDependency);
    } else {
      this.currentVarBitGen.next().value;
    }
  }

  const componentElementSetter = function (
    this: LivonComponentState,
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
    this: LivonComponentState,
    enhancementFunc: () => void
  ) {
    this.__livon_apply_enhancement = enhancementFunc;
  }.bind(this);

  const setAfterMount = function (
    this: LivonComponentState,
    afterMount: () => void
  ) {
    this.__livon_after_mount = afterMount;
  }.bind(this);

  const setAfterUnmount = function (
    this: LivonComponentState,
    afterUnmount: () => void
  ) {
    this.__livon_destroy = afterUnmount;
  }.bind(this);

  const mount = function (
    this: LivonComponentState,
    elm: HTMLElement
  ): LivonComponentState {
    if (this.isMounted) throw new Error("Component is already mounted");
    elm.innerHTML = `<${this.internalElement.topElmTag} ${Object.keys(
      this.internalElement.topElmAttr
    )
      .map((key) => `${key}="${this.internalElement.topElmAttr[key]}"`)
      .join(" ")}>${this.internalElement.innerHtml}</${
      this.internalElement.topElmTag
    }>`;
    this.componentElm = elm.firstElementChild as HTMLElement;
    this.__livon_apply_enhancement();
    this.__livon_after_mount();
    this.isMounted = true;
    _updateComponent(() => {});
    return this;
  }.bind(this);

  const insert = function (
    this: LivonComponentState,
    elm: HTMLElement,
    anchor: HTMLElement | null
  ): LivonComponentState {
    if (this.isMounted) throw new Error("Component is already mounted");
    this.componentElm = _createDomElementFromLivonElement(this.internalElement);
    elm.insertBefore(this.componentElm, anchor);
    this.__livon_apply_enhancement();
    this.__livon_after_mount();
    this.isMounted = true;
    return this;
  }.bind(this);

  const __unmount = function (this: LivonComponentState) {
    if (!this.isMounted) throw new Error("Component is not mounted");
    this.componentElm!.remove();
    this.isMounted = false;
    this.resetDependecies.forEach((r) => r());
    this.__livon_destroy();
  }.bind(this);

  const _updateComponent = function (
    this: LivonComponentState,
    updateFunc: () => void
  ) {
    this.__livon_update = (() => {
      if (!this.updatedFlag) return;
      this.updateComponentFuncs[0].forEach((f) => f());
      this.updateComponentFuncs[1].forEach((f) => f());
      updateFunc.call(this);
      this.updatedFlag = false;
      this.valUpdateMap = [0];
      this.blkUpdateMap = {};
    }).bind(this);
  }.bind(this);

  const createReactive = function <T>(this: LivonComponentState, v: T) {
    return new valueObj<T>(
      v,
      this,
      this.compSymbol,
      this.currentVarBitGen.next().value
    );
  }.bind(this);

  const createIfBlock = function (
    this: LivonComponentState,
    ifBlocks: [
      name: string | (() => string),
      livonElement: () => LivonInternalElement,
      condition: () => boolean,
      postRender: () => void,
      ifCtx: string[],
      forCtx: string[],
      depBit: number | number[],
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
      livonElement,
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
          const componentElm = _createDomElementFromLivonElement(
            livonElement()
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
        if (bitAnd(this.valUpdateMap, depBit)) {
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
        const parentBlockName = indices
          ? `${ifCtxUnderFor[ifCtxUnderFor.length - 1]}-${indices}`
          : ifCtxUnderFor[ifCtxUnderFor.length - 1];
        if (
          this.ifBlockStates[parentBlockName] &&
          condition() &&
          !this.ifBlockStates[name]
        ) {
          this.ifBlocks[name].renderer();
        }
      }

      if (this.forBlocks[forCtx[forCtx.length - 1]]) {
        this.forBlocks[forCtx[forCtx.length - 1]].cleanUp.push(() => {
          [name, ...this.ifBlocks[name].childs].forEach((child) => {
            if (this.ifBlocks[child]) {
              this.ifBlocks[child].cleanup.forEach((f) => f());
              this.ifBlocks[child].cleanup = [];
            }
          });
        });
      }
    }
    this.blkUpdateMap = {};
  }.bind(this);

  const renderIfBlock = function (this: LivonComponentState, name: string) {
    if (!this.ifBlocks[name]) return;
    this.ifBlocks[name].renderer();
  }.bind(this);

  const getElmRefs = function (
    this: LivonComponentState,
    ids: string[],
    preserveId: number | number[],
    refLocation: number | number[] = 0
  ): void {
    const boolMap = bitMapToBoolArr(preserveId);
    ids.forEach(
      function (this: LivonComponentState, id: string, index: number) {
        const e = document.getElementById(id)!;
        if (boolMap[index]) {
          e.removeAttribute("id");
        }
        const newRefLocation = addNumberToArrayInitial(refLocation, index);
        setNestedArrayValue(this.refMap, newRefLocation, e);
      }.bind(this)
    );
  }.bind(this);

  const addEvListener = function (
    this: LivonComponentState,
    args: [number | number[], string, EventListener][]
  ) {
    for (const [elmIdx, evName, evFunc] of args) {
      const target = getNestedArrayValue(this.refMap, elmIdx) as HTMLElement;
      target.addEventListener(evName, evFunc);
    }
  }.bind(this);

  const createForBlock = function (
    this: LivonComponentState,
    forBlocksConfig: [
      forBlockId: string,
      renderItem: (
        item: unknown,
        index: number,
        indices: number[]
      ) => LivonInternalElement,
      getDataArray: () => unknown[],
      afterRenderHook: (
        item: unknown,
        index: number,
        indices: number[]
      ) => void,
      ifCtxUnderFor: string[],
      forCtx: string[],
      updateFlag: number | number[],
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
        if (!Array.isArray(items)) {
          throw new Error(`Items should be an array but got ${typeof items}`);
        }
        items.forEach((item, index) => {
          const fullIndices = [...parentIndices, index];
          const livonElm = renderItem(item, index, fullIndices);
          const domElm = _createDomElementFromLivonElement(livonElm);
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

      let oldItems = getDataArray();

      this.updateComponentFuncs[0].push(
        (() => {
          if (bitAnd(this.valUpdateMap, updateFlag)) {
            const newItems = getDataArray();
            // FIXME: Improve the logic to handle updates properly
            if (diffDetected(oldItems, newItems)) {
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
    this: LivonComponentState,
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
    this: LivonComponentState,
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
        const valueUpdated = bitAnd(this.valUpdateMap, depBit);
        if (!valueUpdated) {
          return;
        }
        const target = getNestedArrayValue(this.refMap, nodeIdx) as Node;
        if (fragmentType === FragmentType.ATTRIBUTE) {
          $$livonReplaceAttr(
            attributeName!,
            textContent(),
            target as HTMLElement
          );
        } else {
          $$livonReplaceText(textContent(), target);
        }
      }).bind(this);
      if (fragmentType === FragmentType.ATTRIBUTE) {
        // Because the determination of the arribute types depends on dynamic values,
        // it is necessary to update the attributes after the initial rendering
        const target = getNestedArrayValue(this.refMap, nodeIdx) as Node;
        $$livonReplaceAttr(
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

  const livonInsertComponent = function (
    this: LivonComponentState,
    componentExport: LivonModuleExports,
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
      const forIndices = indices ? indices.slice(0, -1) : null;
      const forBlockName = forIndices?.length
        ? `${latestCtx}-${forIndices}`
        : latestCtx;
      const ifBlockName = indices ? `${latestCtx}-${indices}` : latestCtx;
      if (this.forBlocks[forBlockName]) {
        this.forBlocks[forBlockName].cleanUp.push(() => {
          componentExport.__unmount();
        });
      } else if (this.ifBlocks[ifBlockName]) {
        this.ifBlocks[ifBlockName].cleanup.push(() => {
          componentExport.__unmount();
        });
      }
    }
  }.bind(this);

  const livonMountComponent = function (
    this: LivonComponentState,
    componentExport: LivonModuleExports,
    parentIdx: number | number[],
    refIdx: number | number[],
    latestCtx: string | null,
    indices: number[] | null
  ) {
    const parentElement = getNestedArrayValue(
      this.refMap,
      parentIdx
    ) as HTMLElement;
    const { componentElm } = componentExport.mount(parentElement);
    setNestedArrayValue(this.refMap, refIdx, componentElm);
    if (latestCtx) {
      const forIndices = indices ? indices.slice(0, -1) : null;
      const forBlockName = forIndices?.length
        ? `${latestCtx}-${forIndices}`
        : latestCtx;
      const ifBlockName = indices ? `${latestCtx}-${indices}` : latestCtx;
      if (this.forBlocks[forBlockName]) {
        this.forBlocks[forBlockName].cleanUp.push(() => {
          componentExport.__unmount();
        });
      } else if (this.ifBlocks[ifBlockName]) {
        this.ifBlocks[ifBlockName].cleanup.push(() => {
          componentExport.__unmount();
        });
      }
    }
  }.bind(this);

  return {
    $$livonSetComponentElement: componentElementSetter,
    $$livonApplyEnhancement: applyEnhancement,
    $$livonAfterMount: setAfterMount,
    $$livonAfterUnmount: setAfterUnmount,
    $$livonReactive: createReactive,
    $$livonCreateIfBlock: createIfBlock,
    $$livonCreateForBlock: createForBlock,
    $$livonRenderIfBlock: renderIfBlock,
    $$livonGetElmRefs: getElmRefs,
    $$livonInsertTextNodes: insertTextNodes,
    $$livonAddEvListener: addEvListener,
    $$livonCreateFragments: createFragments,
    $$livonInsertComponent: livonInsertComponent,
    $$livonMountComponent: livonMountComponent,
    $$livonComponentReturn: {
      mount,
      insert,
      __unmount,
    } as LivonModuleExports,
  };
};

export function $$livonEscapeHtml(text: any): string {
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

export function $$livonReplaceText(content: any, elm: Node) {
  elm.textContent = $$livonEscapeHtml(content);
}

// export function $$livonReplaceInnerHtml(content: any, elm: HTMLElement) {
//   elm.innerHTML = $$livonEscapeHtml(content);
// }

export function $$livonReplaceAttr(
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
  } else if (typeof content === "object") {
    const attrStr = Object.keys(content)
      .filter((k) => content[k])
      .join(" ");
    elm.setAttribute(key, attrStr);
  } else {
    if (content === undefined && elm.hasAttribute(key)) {
      elm.removeAttribute(key);
      return;
    }
    (elm as any)[key] = String(content);
  }
}

export function $$createLivonElement(
  innerHtml: string,
  topElmTag: string,
  topElmAttr: { [key: string]: string } = {}
): LivonInternalElement {
  return {
    innerHtml,
    topElmTag,
    topElmAttr,
  };
}

const _createDomElementFromLivonElement = function (
  livonElement: LivonInternalElement
): HTMLElement {
  const componentElm = document.createElement(livonElement.topElmTag);
  Object.keys(livonElement.topElmAttr).forEach((key) => {
    componentElm.setAttribute(key, livonElement.topElmAttr[key]);
  });
  componentElm.innerHTML = livonElement.innerHtml;
  return componentElm;
};

export const $$livonCreateNonReactive = function <T>(
  this: LivonComponentState,
  v: T
) {
  return new valueObj<T>(v);
};

// const _shouldRender = (
//   blockRendering: boolean,
//   bitValue: number,
//   bitPosition: number
// ): boolean => {
//   // Get the bit at the specified position (1-based index, so subtract 1)
//   const isBitSet = (bitValue & bitPosition) > 0;

//   // Compare the block rendering status with the bit status
//   return blockRendering !== Boolean(isBitSet);
// };

type Fragment = [
  content: [textContent: () => string, attributeName?: string],
  nodeIdx: number[] | number,
  depBit: number | number[],
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

function bitMapToBoolArr(bitMap: number | number[]): boolean[] {
  if (typeof bitMap === "number") {
    return Array.from({ length: 31 }, (_, i) => (bitMap & (1 << i)) !== 0);
  } else {
    return bitMap
      .map((v) => bitMapToBoolArr(v))
      .reduce((acc, val) => acc.concat(val), []);
  }
}

// A function to perform bitwise "&" operation on number[] and number[]
function bitAnd(_a: number | number[], _b: number | number[]): boolean {
  const length = Math.max(
    typeof _a === "number" ? 1 : _a.length,
    typeof _b === "number" ? 1 : _b.length
  );

  const a = fillArrayWithZero(_a, length);
  const b = fillArrayWithZero(_b, length);

  return a.reduce((acc, val, i) => {
    return acc || (val & b[i]) !== 0;
  }, false);
}

// A function to perform bitwise "|=" operation on number[] and number[]
function bitOrAssign(
  target: number | number[],
  source: number | number[]
): void {
  const length = Math.max(
    typeof target === "number" ? 1 : target.length,
    typeof source === "number" ? 1 : source.length
  );

  const targetArr = fillArrayWithZero(target, length);
  const sourceArr = fillArrayWithZero(source, length);

  for (let i = 0; i < length; i++) {
    targetArr[i] |= sourceArr[i];
  }

  if (typeof target === "number") {
    (target as any) = targetArr[0];
  } else {
    for (let i = 0; i < length; i++) {
      target[i] = targetArr[i];
    }
  }
}

// If the lengths of the arrays do not match, add 0 to the shorter array to match the length
function fillArrayWithZero(arr: number[] | number, length: number): number[] {
  let array = typeof arr === "number" ? [arr] : arr;
  while (array.length < length) {
    array.push(0);
  }
  return array;
}

function* bitArrayGenerator(): Generator<number[]> {
  const bitWidth = 31;
  let exp = 0;
  while (true) {
    const digitIndex = Math.floor(exp / bitWidth);
    const bitIndex = exp % bitWidth;
    const out = new Array(digitIndex + 1).fill(0);

    out[digitIndex] = 1 << bitIndex;
    yield out;
    exp++;
  }
}

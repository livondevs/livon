import { isReactive } from "../reactivity";

export type ComponentDeclaration = (args?: {
  [key: string]: any;
}) => LunasModuleExports;

export type LunasModuleExports = {
  mount: (elm: HTMLElement) => LunasComponentState;
  insert: (elm: HTMLElement, anchor: HTMLElement | null) => LunasComponentState;
  __unmount: () => void;
};

enum BlockType {
  IF = "IF",
  FOR = "FOR",
}

export type UpdateBlockFuncs = {
  name: string;
  type: BlockType;
  updateFuncs: (() => void)[];
}[];

export type LunasComponentState = {
  updatedFlag: boolean;
  valUpdateMap: number[];
  internalElement: LunasInternalElement;
  currentVarBitGen: Generator<number[]>;
  ifBlocks: {
    [key: string]: {
      renderer: () => void;
      context: string[];
      forBlk: string | null;
      condition: () => boolean;
      cleanup: (() => void)[];
      childs: string[];
      nextForBlocks: string[];
    };
  };
  ifBlockStates: Record<string, boolean>;
  blkUpdateMap: Record<string, boolean>;
  updateComponentFuncs: ((() => void) | undefined)[][];
  updateBlockFuncs: UpdateBlockFuncs;
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
    [key: string]: {
      cleanUp: (() => void)[];
      childs: string[];
      renderer: () => void;
    };
  };

  refMap: RefMap;
};

type LunasInternalElement = {
  innerHtml: string;
  topElmTag: string;
  topElmAttr: { [key: string]: string };
};

type NestedArray<T> = (T | NestedArray<T>)[];

type FragmentFunc = (item?: unknown, indices?: number[]) => Fragment[];

export class valueObj<T> {
  private _v: T;
  private proxy: T;
  // Dependencies map: key is a symbol, value is a tuple of [LunasComponentState, number[]]
  dependencies: { [key: symbol]: [LunasComponentState, number[]] } = {};

  constructor(
    initialValue: T,
    componentObj?: LunasComponentState,
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
      if (!componentObj.updatedFlag && componentObj.__lunas_update) {
        Promise.resolve().then(componentObj.__lunas_update.bind(componentObj));
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
  addDependency(componentObj: LunasComponentState, symbolIndex: number[]) {
    this.dependencies[componentObj.compSymbol] = [componentObj, symbolIndex];
    return {
      removeDependency: () => {
        delete this.dependencies[componentObj.compSymbol];
      },
    };
  }

  addToCurrentDependency(
    componentObj: LunasComponentState,
    symbolIndex: number[]
  ) {
    const currentDep = Object.getOwnPropertySymbols(this.dependencies).find(
      (key) => key === componentObj.compSymbol
    );
    const currentSymbolIndex = currentDep
      ? this.dependencies[currentDep][1]
      : null;
    if (currentSymbolIndex) {
      const maskedSymbolIndex = bitCombine(currentSymbolIndex, symbolIndex);
      this.dependencies[componentObj.compSymbol] = [
        componentObj,
        maskedSymbolIndex,
      ];
    } else {
      this.dependencies[componentObj.compSymbol] = [componentObj, symbolIndex];
    }
  }
}

export const $$lunasInitComponent = function (
  this: LunasComponentState,
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
  this.updateBlockFuncs = [];
  this.forBlocks = {};
  this.__lunas_after_mount = () => {};
  this.__lunas_destroy = () => {};

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

  const getElm = function (
    this: LunasComponentState,
    location: number | number[]
  ) {
    return getNestedArrayValue(this.refMap, location);
  }.bind(this);

  const setImportVars = function (this: LunasComponentState, items: unknown[]) {
    for (const item of items) {
      if (item instanceof valueObj) {
        const { removeDependency } = item.addDependency(
          this,
          this.currentVarBitGen.next().value
        );
        this.resetDependecies.push(removeDependency);
      } else if (isReactive(item)) {
        const { removeDependency } = item.addDependency(
          this,
          this.currentVarBitGen.next().value
        );
        this.resetDependecies.push(removeDependency);
      } else {
        this.currentVarBitGen.next().value;
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
    _updateComponent(() => {});
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
      this.updateComponentFuncs[0].forEach((f) => f && f());
      const forBlockIds = this.updateBlockFuncs.map((blk) => blk.name);

      const funcsSnapshot: { [key: string]: (() => void)[] } = {};
      for (const id of forBlockIds) {
        funcsSnapshot[id] = this.updateBlockFuncs
          .find((blk) => blk.name === id)!
          .updateFuncs.slice();
      }

      for (const oldKey of forBlockIds) {
        const funcs = this.updateBlockFuncs.find(
          (blk) => blk.name === oldKey
        )!.updateFuncs;
        for (const func of funcs) {
          if (funcsSnapshot[oldKey].indexOf(func) !== -1) {
            func();
          }
        }
      }
      this.updateComponentFuncs[1].forEach((f) => f && f());
      updateFunc.call(this);
      this.updatedFlag = false;
      this.valUpdateMap = [0];
      this.blkUpdateMap = {};
    }).bind(this);
  }.bind(this);

  const createReactive = function <T>(this: LunasComponentState, v: T) {
    return new valueObj<T>(
      v,
      this,
      this.compSymbol,
      this.currentVarBitGen.next().value
    );
  }.bind(this);

  const createIfBlock = function (
    this: LunasComponentState,
    ifBlocks: [
      forBlockId: string | (() => string),
      lunasElement: () => LunasInternalElement,
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
      const ifBlockId = typeof getName === "function" ? getName() : getName;
      setNestedArrayValue(this.refMap, mapOffset, undefined);
      this.ifBlocks[ifBlockId] = {
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
              indices,
              [...ifCtxUnderFor, ifBlockId],
              forCtx[forCtx.length - 1]
            );
          }
          this.ifBlockStates[ifBlockId] = true;
          this.blkUpdateMap[ifBlockId] = true;
          Object.values(this.ifBlocks).forEach((blk) => {
            if (blk.context.includes(ifBlockId)) {
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
        nextForBlocks: [],
      };

      ifCtxUnderFor.forEach((ctx) => {
        const parentBlockName = indices ? `${ctx}-${indices}` : ctx;
        this.ifBlocks[parentBlockName].childs.push(ifBlockId);
      });

      const updateFunc = (() => {
        if (bitAnd(this.valUpdateMap, depBit)) {
          const shouldRender = condition();
          const rendered = !!this.ifBlockStates[ifBlockId];
          const parentRendered = ifCtxUnderFor.every(
            (ctx) => this.ifBlockStates[indices ? `${ctx}-${indices}` : ctx]
          );
          if (shouldRender && !rendered && parentRendered) {
            this.ifBlocks[ifBlockId].renderer();
            this.ifBlocks[ifBlockId].nextForBlocks.forEach((blkName) => {
              const forBlk = this.forBlocks[blkName];
              if (forBlk) {
                forBlk.renderer();
              }
            });
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

            delete this.ifBlockStates[ifBlockId];

            [ifBlockId, ...this.ifBlocks[ifBlockId].childs].forEach((child) => {
              if (this.ifBlocks[child]) {
                this.ifBlocks[child].cleanup.forEach((f) => f());
                this.ifBlocks[child].cleanup = [];
              }
            });
          }
        }
      }).bind(this);

      if (!this.updateBlockFuncs.find((blk) => blk.name === ifBlockId)) {
        this.updateBlockFuncs.push({
          name: ifBlockId,
          type: BlockType.IF,
          updateFuncs: [],
        });
      }
      this.updateBlockFuncs
        .find((blk) => blk.name === ifBlockId)!
        .updateFuncs.push(updateFunc);

      const latestForName = forCtx[forCtx.length - 1];
      if (latestForName) {
        const cleanUpFunc = (() => {
          this.updateBlockFuncs.find(
            (blk) => blk.name === ifBlockId
          )!.updateFuncs = [];
        }).bind(this);
        const popedIndices = copyAndPopArray(indices!);
        const latestForNameWithIndices =
          popedIndices.length > 0
            ? `${latestForName}-${popedIndices}`
            : latestForName;
        this.forBlocks[latestForNameWithIndices]!.cleanUp.push(cleanUpFunc);
      }

      if (ifCtxUnderFor.length === 0) {
        condition() && this.ifBlocks[ifBlockId].renderer();
      } else {
        const parentBlockName = indices
          ? `${ifCtxUnderFor[ifCtxUnderFor.length - 1]}-${indices}`
          : ifCtxUnderFor[ifCtxUnderFor.length - 1];
        if (
          this.ifBlockStates[parentBlockName] &&
          condition() &&
          !this.ifBlockStates[ifBlockId]
        ) {
          this.ifBlocks[ifBlockId].renderer();
        }
      }

      if (this.forBlocks[forCtx[forCtx.length - 1]]) {
        this.forBlocks[forCtx[forCtx.length - 1]].cleanUp.push(() => {
          [ifBlockId, ...this.ifBlocks[ifBlockId].childs].forEach((child) => {
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

  const renderIfBlock = function (this: LunasComponentState, name: string) {
    if (!this.ifBlocks[name]) return;
    this.ifBlocks[name].renderer();
  }.bind(this);

  const getElmRefs = function (
    this: LunasComponentState,
    ids: string[],
    preserveId: number | number[],
    refLocation: number | number[] = 0
  ): void {
    const boolMap = bitMapToBoolArr(preserveId);
    ids.forEach(
      function (this: LunasComponentState, id: string, index: number) {
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
      forBlockId: string | (() => string),
      renderItem: (item: unknown, indices: number[]) => LunasInternalElement,
      getDataArray: () => unknown[],
      afterRenderHook: (item: unknown, indices: number[]) => void,
      ifCtxUnderFor: string[],
      forCtx: string[],
      prevIfCtx: string | null,
      updateFlag: number | number[],
      parentIndices: number[],
      mapInfo: [mapOffset: number, mapLength: number],
      refIdx: [
        parentElementIndex: number | number[],
        refElementIndex?: number | number[]
      ],
      fragment?: FragmentFunc
    ][],
    indices?: number[]
  ): void {
    for (const config of forBlocksConfig) {
      const [
        getName,
        renderItem,
        getDataArray,
        afterRenderHook,
        ifCtxUnderFor,
        forCtx,
        prevIfCtx,
        updateFlag,
        parentIndices,
        [mapOffset, mapLength],
        [parentElementIndex, refElementIndex],
        fragmentFunc,
      ] = config;
      const forBlockId = typeof getName === "function" ? getName() : getName;
      const blkName = indices ? `${prevIfCtx}-${indices}` : prevIfCtx;
      if (prevIfCtx && this.ifBlocks[blkName!]) {
        this.ifBlocks[blkName!].nextForBlocks.push(forBlockId);
      }

      forCtx.forEach((ctx) => {
        const allCtxPatterns = [];
        const copiedIndices = indices ? indices.slice() : [];
        while (true) {
          allCtxPatterns.push(
            copiedIndices.length > 0 ? `${ctx}-${copiedIndices}` : ctx
          );
          copiedIndices.pop();
          if (!copiedIndices || copiedIndices.length === 0) {
            break;
          }
        }
        allCtxPatterns.forEach((ctx) => {
          this.ifBlocks[ctx] && this.ifBlocks[ctx].childs.push(forBlockId);
        });
      });

      let oldItems = deepCopy(getDataArray());

      const renderForBlock = ((items: unknown[]) => {
        const containerElm = getNestedArrayValue(
          this.refMap,
          parentElementIndex
        ) as HTMLElement;
        const insertionPointElm = getNestedArrayValue(
          this.refMap,
          refElementIndex
        ) as HTMLElement;
        if (!(items != null && typeof items[Symbol.iterator] === "function")) {
          throw new Error(`Items should be an iterable object`);
        }
        Array.from(items).forEach((item, index) => {
          const fullIndices = [...parentIndices, index];
          const lunasElm = renderItem(item, fullIndices);
          const domElm = _createDomElementFromLunasElement(lunasElm);
          setNestedArrayValue(this.refMap, [mapOffset, ...fullIndices], domElm);
          containerElm.insertBefore(domElm, insertionPointElm);
          afterRenderHook?.(item, fullIndices);
          if (fragmentFunc) {
            const fragments = fragmentFunc(item, fullIndices);
            createFragments(fragments, indices, ifCtxUnderFor, forBlockId);
          }
        });
        oldItems = deepCopy(getDataArray());
      }).bind(this);

      const toBeRendered = () => {
        return (
          !prevIfCtx ||
          [prevIfCtx].every(
            (ctx) => this.ifBlockStates[indices ? `${ctx}-${indices}` : ctx]
          )
        );
      };

      this.forBlocks[forBlockId] = {
        cleanUp: [],
        childs: [],
        renderer: () => renderForBlock(getDataArray()),
      };

      const updateFunc = (() => {
        if (!toBeRendered()) {
          return;
        }

        if (bitAnd(this.valUpdateMap, updateFlag)) {
          const newItems = Array.from(getDataArray());
          if (diffDetected(oldItems, newItems)) {
            oldItems.forEach((_item, i) => {
              const rs = resetMap(
                this.refMap,
                [mapOffset, ...parentIndices, i],
                mapLength
              );
              for (const r of rs) {
                if (r instanceof HTMLElement) {
                  r.remove();
                }
              }
            });
            if (this.forBlocks[forBlockId]) {
              const { cleanUp, childs } = this.forBlocks[forBlockId];
              cleanUp.forEach((f) => f());
              this.forBlocks[forBlockId].cleanUp = [];
              childs.forEach((child) => {
                if (this.forBlocks[child]) {
                  this.forBlocks[child].cleanUp.forEach((f) => f());
                  this.forBlocks[child].cleanUp = [];
                }
              });
            }
            renderForBlock(newItems);
          }
        }
      }).bind(this);

      if (!this.updateBlockFuncs.find((blk) => blk.name === forBlockId)) {
        this.updateBlockFuncs.push({
          name: forBlockId,
          type: BlockType.FOR,
          updateFuncs: [],
        });
      }
      const forBlock = this.updateBlockFuncs.find(
        (blk) => blk.name === forBlockId
      );
      if (forBlock) {
        forBlock.updateFuncs.push(updateFunc);
      }

      const latestForName = forCtx[forCtx.length - 1];
      if (latestForName) {
        const cleanUpFunc = (() => {
          this.updateBlockFuncs.find(
            (blk) => blk.name === forBlockId
          )!.updateFuncs = [];
          const newIndices = copyAndPopArray(indices!);
          const latestForNameWithIndices =
            newIndices.length > 0
              ? `${latestForName}-${newIndices}`
              : latestForName;
          const childs = this.forBlocks[latestForNameWithIndices].childs;
          childs.forEach((child) => {
            if (this.forBlocks[child]) {
              this.forBlocks[child].cleanUp.forEach((f) => f());
              this.updateBlockFuncs.find(
                (blk) => blk.name === forBlockId
              )!.updateFuncs = [];
              this.forBlocks[child].cleanUp = [];
              this.forBlocks[child].childs = [];
            }
          });
        }).bind(this);
        const popedIndices = copyAndPopArray(indices!);
        const latestForNameWithIndices =
          popedIndices.length > 0
            ? `${latestForName}-${popedIndices}`
            : latestForName;
        this.forBlocks[latestForNameWithIndices]!.cleanUp.push(cleanUpFunc);
      }

      if (!toBeRendered()) {
        return;
      }

      renderForBlock(getDataArray());
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
    indices: number[] | undefined,
    ifCtx?: string[],
    latestForName?: string
  ) {
    for (const [
      [textContent, attributeName, defaultValue],
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
          $$lunasReplaceAttr(
            attributeName!,
            textContent(),
            defaultValue,
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
          defaultValue,
          target as HTMLElement
        );
      }
      this.updateComponentFuncs[1].push(fragmentUpdateFunc);
      if (latestForName) {
        const cleanUpFunc = (() => {
          const idx = this.updateComponentFuncs[1].indexOf(fragmentUpdateFunc);
          this.updateComponentFuncs[1].splice(idx, 1);
        }).bind(this);
        const popedIndices = indices ? copyAndPopArray(indices) : [];
        const latestForNameWithIndices =
          popedIndices.length > 0
            ? `${latestForName}-${popedIndices}`
            : latestForName;
        console.log("latestForNameWithIndices", latestForNameWithIndices);
        this.forBlocks[latestForNameWithIndices]!.cleanUp.push(cleanUpFunc);
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

  const lunasMountComponent = function (
    this: LunasComponentState,
    componentExport: LunasModuleExports,
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

  const watch = function (
    this: LunasComponentState,
    dependingVars: unknown[],
    func: () => void
  ) {
    // Create a combined dependency bit
    let combinedBits: number[] = [0];
    for (const depVar of dependingVars) {
      if (depVar instanceof valueObj) {
        const bit = this.currentVarBitGen.next().value;
        bitOrAssign(combinedBits, bit);
        depVar.addToCurrentDependency(this, bit);
      }
    }
    // Add an update function that calls func when any dependency changes
    const updateFunc = (() => {
      if (bitAnd(this.valUpdateMap, combinedBits)) {
        func();
      }
    }).bind(this);
    this.updateComponentFuncs[0].push(updateFunc);
  }.bind(this);

  return {
    $$lunasGetElm: getElm,
    $$lunasSetImportVars: setImportVars,
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
    $$lunasWatch: watch,
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
  defaultValue: string | undefined,
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
    let attrVal = defaultValue ? `${defaultValue} ` : "";
    attrVal += Object.keys(content)
      .filter((k) => content[k])
      .join(" ");
    elm.setAttribute(key, attrVal);
  } else {
    if (content === undefined && elm.hasAttribute(key)) {
      elm.removeAttribute(key);
      return;
    }
    (elm as any)[key] = String(content);
  }
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
  content: [
    textContent: () => string,
    attributeName?: string,
    defaultValue?: string
  ],
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

function bitCombine(_a: number | number[], _b: number | number[]): number[] {
  const length = Math.max(
    typeof _a === "number" ? 1 : _a.length,
    typeof _b === "number" ? 1 : _b.length
  );

  const a = fillArrayWithZero(_a, length);
  const b = fillArrayWithZero(_b, length);

  const result = new Array<number>(length);
  for (let i = 0; i < length; i++) {
    result[i] = a[i] | b[i];
  }
  return result;
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

function resetMap<T>(
  arr: NestedArray<T>,
  mapLocation: number[],
  length: number
): T[] {
  const results: T[] = [];
  let copied = deepCopy(mapLocation); // deep copy the mapLocation
  for (let i = 0; i < length; i++) {
    let target: any = arr;
    for (let i = 0; i < copied.length - 1; i++) {
      target = target[copied[i]];
    }
    const lastIndex = copied[copied.length - 1];
    const result = target[lastIndex];
    results.push(result);
    target[lastIndex] = undefined;

    copied = addNumberToArrayInitial(copied, 1);
  }

  return results;
}

function deepCopy<T>(data: T): T {
  if (data != null && typeof data === "object") {
    // Check if data is an iterator (has a next method)
    if (typeof (data as any).next === "function") {
      return deepCopy(Array.from(data as unknown as Iterable<unknown>)) as T;
    } else if (Array.isArray(data)) {
      return data.map((item) => deepCopy(item)) as unknown as T;
    } else {
      const result: any = {};
      for (const key in data) {
        if (Object.prototype.hasOwnProperty.call(data, key)) {
          result[key] = deepCopy((data as any)[key]);
        }
      }
      return result;
    }
  }
  return data;
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

function copyAndPopArray(arr: number[]): number[] {
  const copy = arr.slice();
  copy.pop();
  return copy;
}

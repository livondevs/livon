import { LunasComponentState, valueObj } from "../engine";

export function isReactive<T>(
  value: T | ReactiveWrapper<T>
): value is ReactiveWrapper<T> {
  return (
    typeof value === "object" &&
    value !== null &&
    "addDependency" in value &&
    typeof value.addDependency === "function" &&
    "addToCurrentDependency" in value &&
    typeof value.addToCurrentDependency === "function"
  );
}

export type ReactiveWrapper<T> = T & {
  addDependency: (
    componentObj: LunasComponentState,
    symbolIndex: number[]
  ) => { removeDependency: () => void };
  addToCurrentDependency: (
    componentObj: LunasComponentState,
    symbolIndex: number[]
  ) => void;
};

export function reactive<T extends object>(
  initial: T,
  componentObj?: LunasComponentState,
  componentSymbol?: symbol,
  symbolIndex: number[] = [0]
): ReactiveWrapper<T> {
  // 1) Create a valueObj instance that wraps the initial value.
  const wrapper = new valueObj<T>(
    initial,
    componentObj,
    componentSymbol,
    symbolIndex
  );
  // 2) Get the generated Proxy (or primitive) reference.
  const proxy = wrapper.v as T;

  // 3) Directly attach the addDependency method to the Proxy object.
  Object.defineProperty(proxy, "addDependency", {
    value: (cObj: LunasComponentState, sIndex: number[]) => {
      return wrapper.addDependency(cObj, sIndex);
    },
    enumerable: false,
    writable: false,
    configurable: false,
  });

  // 4) Likewise, add addToCurrentDependency if needed.
  Object.defineProperty(proxy, "addToCurrentDependency", {
    value: (cObj: LunasComponentState, sIndex: number[]) => {
      wrapper.addToCurrentDependency(cObj, sIndex);
    },
    enumerable: false,
    writable: false,
    configurable: false,
  });

  return proxy as ReactiveWrapper<T>;
}

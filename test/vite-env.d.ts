/// <reference types="vite/client" />
declare module "*.lun" {
  const src: (args?: { [key: string]: any }) => {
    mount: (el: HTMLElement) => void;
    insertBefore: (el: HTMLElement, ref: HTMLElement) => void;
    __unmount: () => void;
  };
  export default src;
}

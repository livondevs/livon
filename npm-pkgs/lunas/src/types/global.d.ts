import { Router } from "../router/index";

declare global {
  interface Lunas {
    router: Router;
    afterMount: (callback: () => void) => void;
    afterUnmount: (callback: () => void) => void;
    watch: (items: unknown[], callback: () => void) => void;
  }

  var Lunas: Lunas;
}

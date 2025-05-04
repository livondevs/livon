import { Router } from "../runtime/router";

declare global {
  interface Lunas {
    router: Router;
    afterMount: (callback: () => void) => void;
    afterUnmount: (callback: () => void) => void;
    watch: (callback: () => void) => void;
  }

  var Lunas: Lunas;
}

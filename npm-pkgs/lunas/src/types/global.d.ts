import { Router } from "../runtime/router";

declare global {
  interface Lunas {
    router: Router;
    afterMount: (callback: () => void) => void;
  }

  var Lunas: Lunas;
}

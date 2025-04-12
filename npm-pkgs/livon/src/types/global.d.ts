import { Router } from "../runtime/router";

declare global {
  interface Livon {
    router: Router;
    afterMount: (callback: () => void) => void;
    afterUnmount: (callback: () => void) => void;
  }

  var Livon: Livon;
}

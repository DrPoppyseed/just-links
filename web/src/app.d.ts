import { Session } from "./lib/types";

declare global {
  namespace App {
    interface Locals {
      session: {
        username?: string;
      }
    }
  }
}

export {}
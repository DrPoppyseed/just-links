import { Session } from "./lib/types";

declare global {
  namespace App {
    interface Locals {
      session: {
        username?: string;
      };
    }

    interface Platform {
      env: {
        VITE_PUBLIC_APP_SERVER_BASE_URL?: string;
        VITE_PUBLIC_USER_AGENT_URL?: string;
      };
    }
  }
}

export {};

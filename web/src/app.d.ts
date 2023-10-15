import { User } from "firebase/auth";
import { Session } from "./lib/types";

declare global {
  namespace App {
    interface Locals {
      user: User | null;
      getSession(): Promise<Session | null>;
    }
  }
  interface PageData {
    session: Session | null;
  }
}

export {}
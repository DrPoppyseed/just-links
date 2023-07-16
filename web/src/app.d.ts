import { User } from "firebase/auth";

declare global {
  namespace App {
    interface Locals {
      user: User | null;
    }
  }
}

export {};

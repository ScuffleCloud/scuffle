// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
    namespace App {
        // interface Error {}
        // interface Locals {}
        // interface PageData {}
        // interface PageState {}
        // interface Platform {}
        interface PageState {
            loginMode?: import("$lib/types").LoginMode;
            userEmail?: string;
        }
    }
}

export {};

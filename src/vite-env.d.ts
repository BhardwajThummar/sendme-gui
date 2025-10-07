/// <reference types="vite/client" />

declare global {
    interface Window {
        FileResolverPlugin?: {
            resolveContentUri(uri: string): string;
        };
    }
}

export { };


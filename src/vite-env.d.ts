/// <reference types="vite/client" />

declare global {
    interface Window {
        FileResolverPlugin?: {
            resolveContentUri(uri: string): string;
            resolveDirectoryUri(uri: string): string;
            resolveDirectoryToPath(uri: string): string;
        };
        DirectoryPickerPlugin?: {
            openDirectoryPicker(): string;
            openFilePicker(multiple: boolean): string;
            getLastDirectoryUri(): string;
            getLastFileUris(): string;
        };
        IOSPickerPlugin?: {
            openDirectoryPicker(): string;
            openFilePicker(multiple: boolean): string;
            getLastResult(): string;
        };
    }
}

export { };


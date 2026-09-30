/// <reference types="vite/client" />

declare global {
    interface Window {
        IOSPickerPlugin?: {
            openDirectoryPicker(): string;
            openFilePicker(multiple: boolean): string;
            getLastResult(): string;
        };
    }
}

export { };


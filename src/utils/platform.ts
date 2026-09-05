import { isAndroid } from './androidPicker';

/**
 * Check if we're running on iOS
 */
export function isIOS(): boolean {
    return typeof navigator !== 'undefined' && /iphone|ipad|ipod/i.test(navigator.userAgent);
}

/**
 * Check if we're running on a mobile platform (Android or iOS), where the
 * native camera scanner plugin is used instead of getUserMedia.
 */
export function isMobile(): boolean {
    return isAndroid() || isIOS();
}

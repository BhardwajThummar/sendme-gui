import { invoke } from '@tauri-apps/api/core';
import { logger } from './logger';

export interface ResolvedFileEntry {
    path: string;
    name: string;
    sizeBytes: number | null;
    originalUri?: string;
}

interface ResolvedContent {
    path: string;
    name: string;
    size: number;
}

function sanitizeName(name: string): string {
    if (!name) {
        return 'file';
    }

    return name.replace(/\s+/g, ' ').trim();
}

function normalizeSelection(selection: string | string[]): string[] {
    return Array.isArray(selection) ? selection : [selection];
}

async function resolveContentUri(uri: string): Promise<ResolvedFileEntry> {
    const resolved = await invoke<ResolvedContent>('resolve_content_uri', { uri });

    return {
        path: resolved.path,
        name: sanitizeName(resolved.name),
        sizeBytes: Number.isFinite(resolved.size) ? resolved.size : null,
        originalUri: uri,
    };
}

async function safeGetFileSize(path: string): Promise<number | null> {
    try {
        const size = await invoke<number>('get_file_size', { path });
        return Number.isFinite(size) ? size : null;
    } catch (error) {
        logger.warn('FileSelection', `Failed to get file size for path: ${path}`, error);
        return null;
    }
}

async function resolveStandardPath(path: string): Promise<ResolvedFileEntry> {
    const name = sanitizeName(path.split(/[/\\]/).pop() || path);
    const sizeBytes = await safeGetFileSize(path);

    return {
        path,
        name,
        sizeBytes,
    };
}

export async function resolveFileEntries(selection: string | string[] | null): Promise<ResolvedFileEntry[]> {
    if (!selection) {
        return [];
    }

    const entries = normalizeSelection(selection);

    if (!entries.length) {
        return [];
    }

    const allResolvedEntries: ResolvedFileEntry[] = [];

    for (const entry of entries) {
        logger.debug('FileSelection', `Processing entry: ${entry}`);

        if (entry.startsWith('content://')) {
            allResolvedEntries.push(await resolveContentUri(entry));
        } else {
            // Standard file path
            logger.info('FileSelection', 'Resolving as standard path');
            const resolved = await resolveStandardPath(entry);
            allResolvedEntries.push(resolved);
        }
    }

    // Remove duplicates by path
    const uniqueByPath = new Map<string, ResolvedFileEntry>();
    for (const item of allResolvedEntries) {
        if (!uniqueByPath.has(item.path)) {
            uniqueByPath.set(item.path, item);
        }
    }

    return Array.from(uniqueByPath.values());
}

import { invoke } from '@tauri-apps/api/core';
import { logger } from './logger';

export interface ResolvedFileEntry {
    path: string;
    name: string;
    sizeBytes: number | null;
    originalUri?: string;
}

interface FileResolverSuccess {
    success: true;
    path: string;
    originalName?: string;
    size?: number;
}

interface FileResolverError {
    success: false;
    error: string;
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
    if (typeof window === 'undefined' || !window.FileResolverPlugin) {
        throw new Error('Android file resolver bridge is not available.');
    }

    const response = window.FileResolverPlugin.resolveContentUri(uri);

    let parsed: FileResolverSuccess | FileResolverError;

    try {
        parsed = JSON.parse(response) as FileResolverSuccess | FileResolverError;
    } catch (error) {
        throw new Error(`Failed to parse resolver response: ${String(error)}`);
    }

    if (!parsed.success) {
        throw new Error(parsed.error || 'Unknown error while resolving file');
    }

    const displayName = sanitizeName(parsed.originalName || uri.split('/').pop() || 'file');
    const sizeBytes = typeof parsed.size === 'number' && !Number.isNaN(parsed.size) ? parsed.size : null;

    // Fall back to backend size check if the resolver could not determine the file size
    const resolvedSize =
        sizeBytes === null
            ? await safeGetFileSize(parsed.path)
            : sizeBytes;

    return {
        path: parsed.path,
        name: displayName,
        sizeBytes: resolvedSize,
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

    const resolved = await Promise.all(
        entries.map(async (entry) => {
            if (entry.startsWith('content://')) {
                return resolveContentUri(entry);
            }

            return resolveStandardPath(entry);
        })
    );

    const uniqueByPath = new Map<string, ResolvedFileEntry>();
    for (const item of resolved) {
        if (!uniqueByPath.has(item.path)) {
            uniqueByPath.set(item.path, item);
        }
    }

    return Array.from(uniqueByPath.values());
}

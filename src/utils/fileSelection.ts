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

interface DirectoryResolverSuccess {
    success: true;
    isDirectory: true;
    files: Array<{
        path: string;
        originalName: string;
        size: number;
    }>;
    totalSize: number;
    fileCount: number;
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

async function resolveDirectoryUri(uri: string): Promise<ResolvedFileEntry[]> {
    if (typeof window === 'undefined' || !window.FileResolverPlugin) {
        throw new Error('Android file resolver bridge is not available.');
    }

    logger.info('FileSelection', `Resolving directory URI: ${uri}`);

    const response = window.FileResolverPlugin.resolveDirectoryUri(uri);

    logger.debug('FileSelection', `Directory resolver response: ${response}`);

    let parsed: DirectoryResolverSuccess | FileResolverError;

    try {
        parsed = JSON.parse(response) as DirectoryResolverSuccess | FileResolverError;
    } catch (error) {
        throw new Error(`Failed to parse resolver response: ${String(error)}`);
    }

    if (!parsed.success) {
        throw new Error(parsed.error || 'Unknown error while resolving directory');
    }

    logger.info('FileSelection', `Directory resolved with ${parsed.files.length} files`);

    // Convert the array of files to ResolvedFileEntry format
    return parsed.files.map(file => ({
        path: file.path,
        name: sanitizeName(file.originalName),
        sizeBytes: file.size,
        originalUri: uri,
    }));
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
            // Check if this is a directory URI (tree URI from SAF)
            // Directory URIs typically contain '/tree/' in the path
            const isTreeUri = entry.includes('/tree/') || entry.includes('com.android.externalstorage.documents/tree');
            logger.info('FileSelection', `Content URI detected. Is tree URI: ${isTreeUri}`);

            if (isTreeUri) {
                // This is a directory URI, resolve all files within
                logger.info('FileSelection', 'Resolving as directory');
                const directoryFiles = await resolveDirectoryUri(entry);
                allResolvedEntries.push(...directoryFiles);
            } else {
                // Single file URI
                logger.info('FileSelection', 'Resolving as single file');
                const resolved = await resolveContentUri(entry);
                allResolvedEntries.push(resolved);
            }
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

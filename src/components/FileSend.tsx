import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { API_CONFIG, FILE_TRANSFER_CONFIG, UI_CONFIG } from "@/config/app.config";
import { resolveFileEntries } from "@/utils/fileSelection";
import { logger } from "@/utils/logger";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Activity,
  CheckCircle,
  Copy,
  File,
  FolderOpen,
  Loader2,
  Plus,
  Upload,
  X,
} from "lucide-react";
import React, { useEffect, useState } from "react";
import QRCode from "qrcode";
import { startBackgroundService, stopBackgroundService } from "@/utils/backgroundService";
import { isAndroid, openAndroidDirectoryPicker, openAndroidFilePicker } from "@/utils/androidPicker";
import { formatPathForDisplay } from "@/utils/pathFormatter";

interface FileInfo {
  name: string;
  size: string;
  path: string;
  displayName?: string; // Optional formatted name for display
}

interface TransferProgressEvent {
  bytes_transferred: number;
  total_bytes: number;
  percentage: number;
  speed_bytes_per_sec: number;
  elapsed_ms: number;
  eta_ms: number;
}

interface SendCompletedEvent {
  success: boolean;
  message: string;
  elapsed_time_ms: number;
  total_bytes: number;
  files_count: number;
}

interface ImportProgressEvent {
  percentage: number;
  current_file: string;
  files_processed: number;
  total_files: number;
}

const FileSend: React.FC = () => {
  const [selectedFiles, setSelectedFiles] = useState<FileInfo[]>([]);
  const [status, setStatus] = useState<
    "idle" | "processing" | "success" | "error" | "sharing"
  >("idle");
  const [statusMessage, setStatusMessage] = useState<string>("");
  const [sendCode, setSendCode] = useState<string>("");
  // A raw blob ticket (no server configured) is far longer than the
  // fixed-length short code a server hands back.
  const isRawTicket = sendCode.length > FILE_TRANSFER_CONFIG.CODE_LENGTH;
  const [qrDataUrl, setQrDataUrl] = useState<string>("");
  const [importProgress, setImportProgress] = useState<number>(0);
  const [importStatus, setImportStatus] = useState<{
    currentFile: string;
    filesProcessed: number;
    totalFiles: number;
  } | null>(null);

  // Live sharing stats when someone is downloading
  const [sharingStats, setSharingStats] = useState<{
    bytesTransferred: string;
    totalBytes: string;
    speed: string;
    activeConnections: number;
  } | null>(null);

  // Transfer progress percentage for progress bar
  const [transferProgress, setTransferProgress] = useState<number>(0);

  useEffect(() => {
    // Set up event listeners for send/upload events
    const setupSendListeners = async () => {
      const unlisteners: (() => void)[] = [];

      // Listen for import progress during file preparation
      unlisteners.push(
        await listen<ImportProgressEvent>("import_progress", (event) => {
          setImportProgress(event.payload.percentage);
          setImportStatus({
            currentFile: event.payload.current_file,
            filesProcessed: event.payload.files_processed,
            totalFiles: event.payload.total_files,
          });
        })
      );

      // Listen for send completion
      unlisteners.push(
        await listen<SendCompletedEvent>("send_completed", (_event) => {
          logger.info("FileSend", "Files prepared successfully");
        })
      );

      // Listen for transfer progress when someone is downloading
      unlisteners.push(
        await listen<TransferProgressEvent>(
          "send_transfer_progress",
          (event) => {
            const { bytes_transferred, total_bytes, speed_bytes_per_sec, percentage } =
              event.payload;

            setStatus("sharing");
            setTransferProgress(percentage);
            setSharingStats({
              bytesTransferred: formatFileSize(bytes_transferred),
              totalBytes: formatFileSize(total_bytes),
              speed: formatFileSize(speed_bytes_per_sec) + "/s",
              activeConnections: 1,
            });
          }
        )
      );

      return () => {
        unlisteners.forEach((unlisten) => unlisten());
      };
    };

    const cleanupPromise = setupSendListeners();

    return () => {
      cleanupPromise.then((cleanup) => cleanup());
    };
  }, []);

  // Generate a QR code for the raw-ticket case (no server configured)
  useEffect(() => {
    if (!isRawTicket) {
      setQrDataUrl("");
      return;
    }
    QRCode.toDataURL(sendCode, { margin: 1, width: 240 })
      .then(setQrDataUrl)
      .catch((error) => logger.error("FileSend", "Failed to generate QR code", error));
  }, [sendCode, isRawTicket]);

  // Format file size for display
  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return bytes + " B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  const appendResolvedFiles = (
    files: { name: string; path: string; size: string }[]
  ) => {
    if (!files.length) {
      return;
    }

    setSelectedFiles((prev) => {
      const existingPaths = new Set(prev.map((file) => file.path));
      const uniqueNewFiles = files.filter(
        (file) => !existingPaths.has(file.path)
      );
      return uniqueNewFiles.length ? [...prev, ...uniqueNewFiles] : prev;
    });
  };

  const processSelection = async (selection: string | string[] | null) => {
    if (!selection) {
      return;
    }

    setStatus("processing");
    setStatusMessage("Getting file information...");

    try {
      const resolved = await resolveFileEntries(selection);

      const formatted = resolved.map((file) => ({
        name: file.name,
        path: file.path,
        size:
          typeof file.sizeBytes === "number"
            ? formatFileSize(file.sizeBytes)
            : "Size unavailable",
      }));

      appendResolvedFiles(formatted);
      setStatus("idle");
      setStatusMessage("");
    } catch (error) {
      logger.error("FileSend", "Error resolving file selection", error);
      setStatus("error");
      setStatusMessage("Error getting file information");
    }
  };

  const handleFileSelect = async () => {
    try {
      let selected: string | string[] | null = null;

      if (isAndroid()) {
        // Use Android-specific file picker
        const uris = await openAndroidFilePicker(true);
        selected = uris;
      } else {
        // Use Tauri dialog for desktop
        selected = await open({
          multiple: true,
          title: "Select Files to Send",
        });
      }

      await processSelection(selected ?? null);
    } catch (error) {
      logger.error("FileSend", "Error selecting files", error);
      setStatus("error");
      setStatusMessage(`Error selecting files: ${error}`);
    }
  };

  const handleDirSelect = async () => {
    try {
      if (isAndroid()) {
        // On Android, use the custom directory picker
        const uri = await openAndroidDirectoryPicker();
        if (!uri) {
          return; // User cancelled
        }

        setStatus("processing");
        setStatusMessage("Processing directory...");

        // Resolve the directory URI to get the local directory path
        if (!window.FileResolverPlugin) {
          throw new Error('FileResolverPlugin not available');
        }

        const response = JSON.parse(window.FileResolverPlugin.resolveDirectoryToPath(uri));

        if (!response.success) {
          throw new Error(response.error || 'Failed to resolve directory');
        }

        // Add the directory as a single entry
        const formatted = {
          name: response.originalName || "Folder",
          displayName: formatPathForDisplay(response.path),
          path: response.path,
          size: typeof response.size === "number" ? formatFileSize(response.size) : "Calculating...",
        };

        appendResolvedFiles([formatted]);
        setStatus("idle");
        setStatusMessage("");

        logger.info("FileSend", `Directory added: ${response.path}`);
      } else {
        // Use Tauri dialog for desktop
        const selected = await open({
          multiple: true,
          directory: true,
          title: "Select Directories to Send",
        });

        if (selected && Array.isArray(selected) && selected.length > 0) {
          await processSelection(selected);
        }
      }
    } catch (error) {
      logger.error("FileSend", "Error selecting directories", error);
      setStatus("error");
      setStatusMessage(`Error selecting files: ${error}`);
    }
  };

  // Remove a file from the selection
  const removeFile = (index: number) => {
    setSelectedFiles((files) => files.filter((_, i) => i !== index));
    if (selectedFiles.length <= 1) {
      setSendCode("");
    }
  };

  // Handle the send button click
  const handleSend = async () => {
    if (selectedFiles.length === 0) {
      setStatus("error");
      setStatusMessage("Please select at least one file");
      return;
    }

    setStatus("processing");
    setStatusMessage("Preparing your files...");

    // Start background service for Android to keep transfer alive
    startBackgroundService('send', selectedFiles.length);

    try {
      // Send all selected file paths
      const filePaths = selectedFiles.map(f => f.path);

      const result = await invoke<string>("send_files_command", {
        filePaths: filePaths,
        verbose: API_CONFIG.VERBOSE_MODE,
      });

      setSendCode(result);
      setStatus("success");
      setStatusMessage("Your files are ready to send!");
      logger.info("FileSend", `Send code generated: ${result}`);
    } catch (error) {
      logger.error("FileSend", "Error sending files", error);
      setStatus("error");
      setStatusMessage(`Error: ${error}`);
      // Stop service on error
      stopBackgroundService();
    }
  };

  // Copy send code to clipboard
  const copyCodeToClipboard = () => {
    navigator.clipboard.writeText(sendCode);
    setStatusMessage("Code copied to clipboard!");

    setTimeout(() => {
      if (status === "success") {
        setStatusMessage("Your files are ready to send!");
      }
    }, UI_CONFIG.CLIPBOARD_NOTIFICATION_DURATION);
  };

  // Reset the send process
  const handleReset = () => {
    setSelectedFiles([]);
    setStatus("idle");
    setStatusMessage("");
    setSendCode("");
  };

  return (
    <div className="w-full">
      {sendCode ? (
        <div className="space-y-6">
          <div className="flex flex-col items-center justify-center">
            <div className="w-16 h-16 rounded-full bg-muted flex items-center justify-center mb-4">
              {status === "sharing" ? (
                <Activity className="h-8 w-8 text-primary animate-pulse" />
              ) : (
                <CheckCircle className="h-8 w-8 text-primary" />
              )}
            </div>
            <h2 className="text-xl font-bold text-center">
              {status === "sharing"
                ? "Sharing in Progress"
                : isRawTicket
                ? "Your Send Ticket"
                : "Your Send Code"}
            </h2>

            {isRawTicket ? (
              <div className="flex flex-col items-center gap-3 my-4 w-full">
                {qrDataUrl && (
                  <img
                    src={qrDataUrl}
                    alt="QR code for send ticket"
                    className="rounded-lg border border-border bg-background p-2"
                  />
                )}
                <p className="break-all text-xs font-mono bg-muted p-3 rounded-lg border border-border w-full text-center">
                  {sendCode}
                </p>
              </div>
            ) : (
              <div className="flex flex-wrap justify-center items-center gap-2 text-2xl font-mono bg-muted p-4 rounded-lg my-4 w-full border border-border">
                {sendCode.split("").map((char, index) => (
                  <span
                    key={index}
                    className="inline-flex items-center justify-center w-10 h-12 rounded-md bg-background border border-border shadow-sm"
                  >
                    {char}
                  </span>
                ))}
              </div>
            )}

            <p className="text-muted-foreground text-sm text-center px-4">
              {status === "sharing"
                ? "Someone is downloading your files..."
                : isRawTicket
                ? "Scan the QR code or copy the ticket to let them download your files"
                : "Send this code to the recipient to let them download your files"}
            </p>
          </div>

          {status === "sharing" && sharingStats && (
            <div className="mx-2 p-4 border border-primary rounded-lg bg-muted space-y-3">
              <h3 className="text-sm font-medium text-primary">
                Live Transfer Stats
              </h3>

              {/* Progress Bar */}
              <div className="space-y-2">
                <div className="flex justify-between items-center text-xs">
                  <span className="text-muted-foreground">Uploading...</span>
                  <span className="font-medium text-primary">
                    {Math.round(transferProgress)}%
                  </span>
                </div>
                <Progress value={transferProgress} className="h-2" />
              </div>

              <div className="grid grid-cols-2 gap-3 text-xs">
                <div>
                  <div className="text-muted-foreground">Progress</div>
                  <div className="font-medium">
                    {sharingStats.bytesTransferred} / {sharingStats.totalBytes}
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Speed</div>
                  <div className="font-medium">{sharingStats.speed}</div>
                </div>
              </div>
            </div>
          )}

          <div className="flex flex-col gap-3 px-2">
            <Button
              onClick={copyCodeToClipboard}
              className="flex items-center justify-center gap-2 w-full"
            >
              <Copy className="h-4 w-4" />
              Copy Code
            </Button>

            <Button variant="outline" onClick={handleReset} className="w-full">
              Send Different Files
            </Button>
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <Button
              variant="outline"
              className="h-auto py-4 flex flex-col items-center gap-2 border-border hover:border-primary hover:bg-muted"
              onClick={handleFileSelect}
              disabled={status === "processing"}
            >
              {status === "processing" ? (
                <Loader2 className="h-5 w-5 animate-spin text-primary" />
              ) : (
                <Upload className="h-6 w-6 text-primary" />
              )}
              <div className="text-center">
                <div className="font-medium text-xs">Select Files</div>
              </div>
            </Button>

            <Button
              variant="outline"
              className="h-auto py-4 flex flex-col items-center gap-2 border-border hover:border-primary hover:bg-muted"
              onClick={handleDirSelect}
              disabled={status === "processing"}
            >
              {status === "processing" ? (
                <Loader2 className="h-5 w-5 animate-spin text-primary" />
              ) : (
                <FolderOpen className="h-6 w-6 text-primary" />
              )}
              <div className="text-center">
                <div className="font-medium text-xs">Select Folders</div>
              </div>
            </Button>
          </div>

          {selectedFiles.length > 0 && (
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-medium">
                  Selected Files ({selectedFiles.length})
                </h3>
              </div>

              <div className="border border-border rounded-lg divide-y divide-border max-h-[200px] overflow-y-auto">
                {selectedFiles.map((file, index) => (
                  <div
                    key={index}
                    className="flex items-center justify-between p-2 hover:bg-muted"
                  >
                    <div className="flex items-center gap-2 flex-1 min-w-0">
                      <File className="h-4 w-4 text-foreground shrink-0" />
                      <div className="min-w-0">
                        <div className="font-medium text-xs truncate">
                          {file.displayName || file.name}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          {file.size}
                        </div>
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6 text-muted-foreground hover:text-destructive shrink-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        removeFile(index);
                      }}
                    >
                      <X className="h-3 w-3" />
                    </Button>
                  </div>
                ))}
              </div>

              <div className="flex flex-col gap-2">
                <Button
                  variant="outline"
                  className="flex items-center gap-2 w-full"
                  onClick={handleFileSelect}
                  disabled={status === "processing"}
                  size="sm"
                >
                  <Plus className="h-3 w-3" />
                  {status === "processing" ? "Processing..." : "Add More Files"}
                </Button>

                <Button
                  className="flex items-center gap-2 w-full"
                  onClick={handleSend}
                  disabled={
                    status === "processing" || selectedFiles.length === 0
                  }
                >
                  {status === "processing" ? (
                    <>
                      <Loader2 className="h-4 w-4 animate-spin" />
                      <span>Preparing...</span>
                    </>
                  ) : (
                    "Generate Send Code"
                  )}
                </Button>
              </div>
            </div>
          )}

          {status === "processing" && (
            <div className="space-y-2 mt-4 p-3 border border-border rounded-md bg-muted">
              <div className="flex justify-between items-center text-xs">
                <span className="text-muted-foreground">
                  Preparing files...
                  {importStatus && ` (${importStatus.filesProcessed}/${importStatus.totalFiles})`}
                </span>
                <span className="font-medium text-primary">
                  {Math.round(importProgress)}%
                </span>
              </div>
              <Progress value={importProgress} className="h-2" />
              {importStatus && importStatus.currentFile && (
                <p className="text-xs text-muted-foreground truncate">
                  Processing: {importStatus.currentFile}
                </p>
              )}
              {statusMessage && (
                <p className="text-xs text-center text-muted-foreground">
                  {statusMessage}
                </p>
              )}
            </div>
          )}

          {status === "error" && statusMessage && (
            <div className="p-3 rounded-md bg-muted border border-destructive">
              <div className="flex items-center gap-2 text-destructive">
                <X className="h-4 w-4 shrink-0" />
                <p className="text-xs">{statusMessage}</p>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default FileSend;

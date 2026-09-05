import { Button } from "@/components/ui/button";
import { logger } from "@/utils/logger";
import { isMobile } from "@/utils/platform";
import { cancel, Format, scan } from "@tauri-apps/plugin-barcode-scanner";
import jsQR from "jsqr";
import { X } from "lucide-react";
import React, { useEffect, useRef, useState } from "react";

interface QrScannerProps {
  onDetected: (content: string) => void;
  onClose: () => void;
}

/**
 * QR scan overlay. On mobile, delegates to the native camera scanner
 * plugin (its own full-screen UI). On desktop, captures the webcam via
 * getUserMedia and decodes frames locally with jsQR.
 */
const QrScanner: React.FC<QrScannerProps> = ({ onDetected, onClose }) => {
  const [error, setError] = useState<string>("");
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const frameRef = useRef<number>(0);

  useEffect(() => {
    if (isMobile()) {
      let cancelled = false;

      scan({ formats: [Format.QRCode] })
        .then((result) => {
          if (!cancelled) onDetected(result.content);
        })
        .catch((err) => {
          logger.error("QrScanner", "Native scan failed", err);
          if (!cancelled) onClose();
        });

      return () => {
        cancelled = true;
        cancel().catch(() => {});
      };
    }

    // Desktop: getUserMedia + jsQR
    let stopped = false;

    const start = async () => {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          video: { facingMode: "environment" },
        });
        streamRef.current = stream;

        if (videoRef.current) {
          videoRef.current.srcObject = stream;
          await videoRef.current.play();
        }

        const tick = () => {
          if (stopped) return;

          const video = videoRef.current;
          const canvas = canvasRef.current;
          if (video && canvas && video.readyState === video.HAVE_ENOUGH_DATA) {
            canvas.width = video.videoWidth;
            canvas.height = video.videoHeight;
            const ctx = canvas.getContext("2d");
            if (ctx) {
              ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
              const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
              const code = jsQR(imageData.data, imageData.width, imageData.height);
              if (code) {
                onDetected(code.data);
                return;
              }
            }
          }
          frameRef.current = requestAnimationFrame(tick);
        };
        frameRef.current = requestAnimationFrame(tick);
      } catch (err) {
        logger.error("QrScanner", "Failed to access camera", err);
        setError("Could not access the camera. Check permissions and try again.");
      }
    };

    start();

    return () => {
      stopped = true;
      cancelAnimationFrame(frameRef.current);
      streamRef.current?.getTracks().forEach((track) => track.stop());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="fixed inset-0 z-50 bg-background/95 flex flex-col items-center justify-center p-4">
      <Button
        variant="outline"
        size="icon"
        onClick={onClose}
        className="absolute top-4 right-4"
        title="Close scanner"
      >
        <X className="h-4 w-4" />
      </Button>

      {isMobile() ? (
        <p className="text-sm text-muted-foreground">Opening camera...</p>
      ) : (
        <div className="w-full max-w-sm space-y-3">
          <video ref={videoRef} className="w-full rounded-lg border border-border" muted playsInline />
          <canvas ref={canvasRef} className="hidden" />
          <p className="text-xs text-center text-muted-foreground">
            Point the camera at the sender's QR code
          </p>
          {error && <p className="text-xs text-center text-destructive">{error}</p>}
        </div>
      )}
    </div>
  );
};

export default QrScanner;

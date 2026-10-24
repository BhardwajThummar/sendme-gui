import { Button } from "@/components/ui/button";
import { logger } from "@/utils/logger";
import jsQR from "jsqr";
import { X } from "lucide-react";
import React, { useEffect, useRef, useState } from "react";

interface QrScannerProps {
  onDetected: (content: string) => void;
  onClose: () => void;
}

/**
 * QR scan overlay. Captures the camera through the web view (getUserMedia) and
 * decodes frames locally with jsQR, on every platform. The native plugin was
 * tried on iOS and on Android: on Android it needs Google Play Services to
 * download its decoder, and on iOS its preview view never became visible even
 * though the camera session and the system's QR decoder were running.
 */
const QrScanner: React.FC<QrScannerProps> = ({ onDetected, onClose }) => {
  const [error, setError] = useState<string>("");
  // Shown until the first camera frame arrives
  const [status, setStatus] = useState<string>("Starting the camera...");
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const frameRef = useRef<number>(0);

  useEffect(() => {
    let stopped = false;

    const start = async () => {
      try {
        if (!navigator.mediaDevices?.getUserMedia) {
          throw new Error("this view has no camera access (mediaDevices missing)");
        }
        const stream = await navigator.mediaDevices.getUserMedia({
          video: {
            facingMode: { ideal: "environment" },
            width: { ideal: 1280 },
            height: { ideal: 720 },
          },
        });
        streamRef.current = stream;

        if (videoRef.current) {
          videoRef.current.srcObject = stream;
          await videoRef.current.play();
        }

        let frames = 0;
        let lastDecode = 0;
        const tick = (now: number) => {
          if (stopped) return;

          const video = videoRef.current;
          const canvas = canvasRef.current;
          if (video && canvas && video.readyState >= 2 && video.videoWidth > 0) {
            if (canvas.width !== video.videoWidth || canvas.height !== video.videoHeight) {
              canvas.width = video.videoWidth;
              canvas.height = video.videoHeight;
            }
            const ctx = canvas.getContext("2d", { willReadFrequently: true });
            if (ctx) {
              // The canvas is the preview. iOS 27's web view did not paint the <video>
              // element showing a live camera stream, but it paints a canvas fine.
              ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
              frames += 1;
              if (frames === 1) setStatus("");
              // Decoding is the expensive part; ten times a second is plenty.
              if (now - lastDecode > 100) {
                lastDecode = now;
                const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
                const code = jsQR(imageData.data, imageData.width, imageData.height, {
                  inversionAttempts: "dontInvert",
                });
                if (code) {
                  onDetected(code.data);
                  return;
                }
              }
            }
          }
          frameRef.current = requestAnimationFrame(tick);
        };
        frameRef.current = requestAnimationFrame(tick);
      } catch (err) {
        logger.error("QrScanner", "Failed to access camera", err);
        const e = err as { name?: string; message?: string };
        setError(
          `Could not access the camera (${e?.name ?? "error"}: ${e?.message ?? String(err)}). Check permissions and try again.`
        );
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

      <div className="w-full max-w-sm space-y-3">
        {/* The video only feeds the canvas; it is kept (not display:none) so it keeps decoding. */}
        <video ref={videoRef} className="absolute h-px w-px opacity-0 pointer-events-none" muted playsInline />
        <canvas ref={canvasRef} className="w-full rounded-lg border border-border" />
        <p className="text-xs text-center text-muted-foreground">
          Point the camera at the sender's QR code
        </p>
        {status && <p className="text-xs text-center text-muted-foreground">{status}</p>}
        {error && <p className="text-xs text-center text-destructive">{error}</p>}
      </div>
    </div>
  );
};

export default QrScanner;

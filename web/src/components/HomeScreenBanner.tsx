import React, { useEffect, useState } from "react";
import { Share, X, PlusSquare } from "lucide-react";

export const HomeScreenBanner: React.FC = () => {
  const [show, setShow] = useState(false);

  useEffect(() => {
    // Detect iOS and check if already running as standalone PWA
    const isIOS =
      /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window as any).MSStream;
    const isStandalone =
      (window.navigator as any).standalone ||
      window.matchMedia("(display-mode: standalone)").matches;

    const dismissed = localStorage.getItem("st_dismiss_pwa_banner") === "true";

    if (isIOS && !isStandalone && !dismissed) {
      setShow(true);
    }
  }, []);

  const handleDismiss = () => {
    localStorage.setItem("st_dismiss_pwa_banner", "true");
    setShow(false);
  };

  if (!show) return null;

  return (
    <div
      style={{
        margin: "12px 16px 0 16px",
        padding: "14px 16px",
        background: "var(--panel2)",
        border: "1px solid var(--accent)",
        borderRadius: "var(--radius-card)",
        position: "relative",
        boxShadow: "0 4px 16px rgba(0, 0, 0, 0.4)",
      }}
    >
      <button
        onClick={handleDismiss}
        style={{
          position: "absolute",
          top: "10px",
          right: "10px",
          background: "transparent",
          color: "var(--muted)",
          padding: "4px",
        }}
        aria-label="Dismiss banner"
      >
        <X size={16} />
      </button>

      <div style={{ display: "flex", gap: "12px", alignItems: "flex-start" }}>
        <div
          style={{
            background: "var(--accent-dim)",
            color: "var(--accent)",
            padding: "8px",
            borderRadius: "10px",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexShrink: 0,
          }}
        >
          <PlusSquare size={22} />
        </div>

        <div style={{ paddingRight: "16px" }}>
          <h4 style={{ fontSize: "14px", fontWeight: 600, margin: "0 0 4px 0" }}>
            Install on iPhone
          </h4>
          <p style={{ fontSize: "12.5px", color: "var(--muted)", margin: 0, lineHeight: 1.4 }}>
            For the best full-screen experience and push notifications, tap the{" "}
            <strong style={{ color: "var(--text)", display: "inline-flex", alignItems: "center", gap: "2px" }}>
              Share <Share size={12} />
            </strong>{" "}
            icon in Safari and select <strong>"Add to Home Screen"</strong>.
          </p>
        </div>
      </div>
    </div>
  );
};

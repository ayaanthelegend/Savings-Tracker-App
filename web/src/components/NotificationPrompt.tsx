import React, { useState, useEffect } from "react";
import { Bell, X } from "lucide-react";
import { registerPushSubscription } from "../lib/supabase";

interface NotificationPromptProps {
  userId: string;
}

export const NotificationPrompt: React.FC<NotificationPromptProps> = ({ userId }) => {
  const [show, setShow] = useState(false);
  const [loading, setLoading] = useState(false);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  useEffect(() => {
    // Check if notifications are supported and permission is currently 'default'
    if (typeof window !== "undefined" && "Notification" in window) {
      const dismissed = localStorage.getItem("st_dismiss_push_prompt") === "true";
      if (Notification.permission === "default" && !dismissed) {
        setShow(true);
      }
    }
  }, []);

  const handleEnable = async () => {
    setLoading(true);
    setStatusMsg(null);

    // Get VAPID public key from env or default
    const vapidKey =
      (import.meta as any).env?.VITE_VAPID_PUBLIC_KEY ||
      "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5NPHI84";

    const res = await registerPushSubscription(userId, vapidKey);
    setLoading(false);

    if (res.success) {
      setStatusMsg("Deadline notifications enabled!");
      setTimeout(() => setShow(false), 2000);
    } else {
      setStatusMsg(res.error || "Permission was not granted.");
      setTimeout(() => setShow(false), 3000);
    }
  };

  const handleDismiss = () => {
    localStorage.setItem("st_dismiss_push_prompt", "true");
    setShow(false);
  };

  if (!show) return null;

  return (
    <div
      style={{
        margin: "12px 16px 0 16px",
        padding: "12px 16px",
        background: "var(--panel2)",
        border: "1px solid var(--border)",
        borderRadius: "var(--radius-card)",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: "12px",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
        <div
          style={{
            background: "rgba(232, 185, 63, 0.15)",
            color: "var(--star)",
            padding: "8px",
            borderRadius: "50%",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <Bell size={18} />
        </div>
        <div>
          <div style={{ fontSize: "13px", fontWeight: 600 }}>Enable iPhone Alerts</div>
          <div style={{ fontSize: "11.5px", color: "var(--muted)" }}>
            {statusMsg || "Get notified when savings plans reach their deadline"}
          </div>
        </div>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
        <button
          onClick={handleEnable}
          disabled={loading}
          style={{
            background: "var(--accent)",
            color: "#ffffff",
            padding: "5px 10px",
            borderRadius: "var(--radius-btn)",
            fontSize: "12px",
            fontWeight: 600,
          }}
        >
          {loading ? "..." : "Enable"}
        </button>

        <button
          onClick={handleDismiss}
          style={{
            background: "transparent",
            color: "var(--muted)",
            padding: "4px",
          }}
          aria-label="Dismiss"
        >
          <X size={15} />
        </button>
      </div>
    </div>
  );
};

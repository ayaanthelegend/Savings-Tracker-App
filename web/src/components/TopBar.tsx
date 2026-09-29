import React from "react";
import { Settings, RefreshCw, ChevronDown, Plus } from "lucide-react";
import { Card } from "../types";

interface TopBarProps {
  cards: Card[];
  selectedCardId: string;
  onSelectCard: (cardId: string) => void;
  syncStatus: "synced" | "syncing" | "offline" | "error";
  onTriggerSync: () => void;
  onOpenSettings: () => void;
  onQuickAdd: () => void;
  tabTitle: string;
}

export const TopBar: React.FC<TopBarProps> = ({
  cards,
  selectedCardId,
  onSelectCard,
  syncStatus,
  onTriggerSync,
  onOpenSettings,
  onQuickAdd,
  tabTitle,
}) => {
  const currentCard = cards.find((c) => c.id === selectedCardId) || cards[0];

  return (
    <header
      style={{
        position: "sticky",
        top: 0,
        zIndex: 50,
        backgroundColor: "var(--panel)",
        borderBottom: "1px solid var(--border)",
        paddingTop: "max(12px, var(--sat))",
        paddingBottom: "12px",
        paddingLeft: "max(16px, var(--sal))",
        paddingRight: "max(16px, var(--sar))",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        backdropFilter: "blur(20px)",
        WebkitBackdropFilter: "blur(20px)",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
        {cards.length > 1 ? (
          <div style={{ position: "relative" }}>
            <select
              value={selectedCardId}
              onChange={(e) => onSelectCard(e.target.value)}
              style={{
                appearance: "none",
                WebkitAppearance: "none",
                background: "var(--panel2)",
                border: "1px solid var(--border)",
                color: "var(--text)",
                padding: "6px 28px 6px 12px",
                borderRadius: "var(--radius-btn)",
                fontSize: "14px",
                fontWeight: 600,
                fontFamily: "var(--font-head)",
                cursor: "pointer",
              }}
            >
              {cards.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name} {c.is_primary ? "★" : ""}
                </option>
              ))}
            </select>
            <ChevronDown
              size={14}
              style={{
                position: "absolute",
                right: "8px",
                top: "50%",
                transform: "translateY(-50%)",
                pointerEvents: "none",
                color: "var(--muted)",
              }}
            />
          </div>
        ) : (
          <h1 style={{ fontSize: "17px", fontWeight: 700, margin: 0 }}>
            {tabTitle}
          </h1>
        )}

        {/* Sync Status Pill */}
        <button
          onClick={onTriggerSync}
          title={`Sync status: ${syncStatus}. Tap to refresh.`}
          style={{
            background: "var(--panel2)",
            border: "1px solid var(--border)",
            borderRadius: "12px",
            padding: "4px 8px",
            display: "inline-flex",
            alignItems: "center",
            gap: "5px",
            fontSize: "11px",
            color: "var(--muted)",
          }}
        >
          {syncStatus === "syncing" ? (
            <RefreshCw size={12} className="spin-icon" style={{ color: "var(--accent)" }} />
          ) : (
            <span
              style={{
                width: "7px",
                height: "7px",
                borderRadius: "50%",
                backgroundColor:
                  syncStatus === "synced"
                    ? "var(--in)"
                    : syncStatus === "offline"
                    ? "var(--muted)"
                    : "var(--out)",
              }}
            />
          )}
          <span style={{ textTransform: "capitalize" }}>
            {syncStatus === "syncing" ? "Syncing..." : syncStatus}
          </span>
        </button>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <button
          onClick={onQuickAdd}
          style={{
            background: "var(--accent)",
            color: "#ffffff",
            padding: "6px 12px",
            borderRadius: "var(--radius-btn)",
            fontSize: "13px",
            fontWeight: 600,
            gap: "4px",
          }}
        >
          <Plus size={16} strokeWidth={2.5} />
          <span>Add</span>
        </button>

        <button
          onClick={onOpenSettings}
          style={{
            background: "transparent",
            color: "var(--muted)",
            padding: "6px",
            borderRadius: "var(--radius-btn)",
          }}
          title="Settings"
        >
          <Settings size={20} />
        </button>
      </div>

      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
        .spin-icon {
          animation: spin 1s linear infinite;
        }
      `}</style>
    </header>
  );
};

import React, { useState } from "react";
import { X, LogOut, RefreshCw, Bell, Plus, CreditCard, Tag, Check, Sparkles } from "lucide-react";
import { Card, Category } from "../types";
import { formatPkr, generateUuid, parsePkrInput, todayIso } from "../lib/utils";
import { registerPushSubscription } from "../lib/supabase";

interface SettingsModalProps {
  userEmail: string;
  userId: string;
  cards: Card[];
  categories: Category[];
  onAddCard: (card: Card) => Promise<void>;
  onAddCategory: (category: Category) => Promise<void>;
  onTriggerSync: () => Promise<void>;
  onLogOut: () => Promise<void>;
  onClose: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  userEmail,
  userId,
  cards,
  categories,
  onAddCard,
  onAddCategory,
  onTriggerSync,
  onLogOut,
  onClose,
}) => {
  const [activeSection, setActiveSection] = useState<"general" | "cards" | "categories">("general");

  // New Card Form
  const [showAddCard, setShowAddCard] = useState(false);
  const [cardName, setCardName] = useState("");
  const [openingBalanceRaw, setOpeningBalanceRaw] = useState("0");
  const [openingDate, setOpeningDate] = useState(todayIso());
  const [isPrimary, setIsPrimary] = useState(false);

  // New Category Form
  const [newCatName, setNewCatName] = useState("");

  // Push notification state
  const [pushStatus, setPushStatus] = useState<string | null>(null);
  const [isSubscribingPush, setIsSubscribingPush] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);

  const handleCreateCard = async (e: React.FormEvent) => {
    e.preventDefault();
    const bal = parsePkrInput(openingBalanceRaw) ?? 0;
    if (!cardName.trim()) return;

    const newCard: Card = {
      id: generateUuid(),
      name: cardName.trim(),
      is_primary: isPrimary,
      opening_balance: bal,
      opening_balance_description: "Opening Balance",
      opening_balance_date: openingDate,
      updated_at: new Date().toISOString(),
      deleted_at: null,
    };

    await onAddCard(newCard);
    setCardName("");
    setOpeningBalanceRaw("0");
    setShowAddCard(false);
  };

  const handleCreateCategory = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newCatName.trim()) return;

    const newCat: Category = {
      id: generateUuid(),
      name: newCatName.trim(),
      is_default: false,
      updated_at: new Date().toISOString(),
      deleted_at: null,
    };

    await onAddCategory(newCat);
    setNewCatName("");
  };

  const handleEnablePush = async () => {
    setIsSubscribingPush(true);
    setPushStatus(null);

    const vapidKey =
      (import.meta as any).env?.VITE_VAPID_PUBLIC_KEY ||
      "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5NPHI84";

    const res = await registerPushSubscription(userId, vapidKey);
    setIsSubscribingPush(false);

    if (res.success) {
      setPushStatus("Push notifications successfully registered for this iPhone!");
    } else {
      setPushStatus(res.error || "Failed to register notifications.");
    }
  };

  const handleManualSync = async () => {
    setIsSyncing(true);
    await onTriggerSync();
    setIsSyncing(false);
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-handle" />

        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
          <h3 style={{ margin: 0, fontSize: "18px" }}>App Settings</h3>
          <button onClick={onClose} style={{ background: "transparent", color: "var(--muted)", padding: "4px" }}>
            <X size={18} />
          </button>
        </div>

        {/* Section Tabs */}
        <div style={{ display: "flex", gap: "6px", marginBottom: "18px" }}>
          {(["general", "cards", "categories"] as const).map((sec) => (
            <button
              key={sec}
              onClick={() => setActiveSection(sec)}
              style={{
                flex: 1,
                padding: "8px",
                fontSize: "12px",
                fontWeight: 600,
                borderRadius: "var(--radius-btn)",
                background: activeSection === sec ? "var(--accent)" : "var(--panel2)",
                color: activeSection === sec ? "#ffffff" : "var(--muted)",
                border: `1px solid ${activeSection === sec ? "var(--accent)" : "var(--border)"}`,
                textTransform: "capitalize",
              }}
            >
              {sec}
            </button>
          ))}
        </div>

        {/* 1. General Section */}
        {activeSection === "general" && (
          <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            {/* User Account info */}
            <div style={{ background: "var(--panel2)", padding: "12px 14px", borderRadius: "var(--radius-btn)", border: "1px solid var(--border)" }}>
              <div style={{ fontSize: "11px", color: "var(--muted)", textTransform: "uppercase" }}>Signed in as</div>
              <div style={{ fontSize: "14px", fontWeight: 600, marginTop: "2px" }}>{userEmail}</div>
            </div>

            {/* Sync Controls */}
            <div>
              <div style={{ fontSize: "13px", fontWeight: 600, marginBottom: "8px", color: "var(--muted)" }}>
                Cloud Sync
              </div>
              <button
                type="button"
                onClick={handleManualSync}
                disabled={isSyncing}
                style={{
                  width: "100%",
                  padding: "10px",
                  background: "var(--panel2)",
                  border: "1px solid var(--border)",
                  borderRadius: "var(--radius-btn)",
                  color: "var(--text)",
                  fontWeight: 600,
                  fontSize: "13px",
                  gap: "8px",
                }}
              >
                <RefreshCw size={15} className={isSyncing ? "spin-icon" : ""} />
                {isSyncing ? "Syncing with Cloud..." : "Sync Now"}
              </button>
            </div>

            {/* iPhone Web Push */}
            <div>
              <div style={{ fontSize: "13px", fontWeight: 600, marginBottom: "8px", color: "var(--muted)" }}>
                iPhone Push Alerts
              </div>
              <div style={{ background: "var(--panel2)", padding: "12px", borderRadius: "var(--radius-btn)", border: "1px solid var(--border)" }}>
                <p style={{ fontSize: "12px", color: "var(--muted)", margin: "0 0 10px 0", lineHeight: 1.4 }}>
                  Web Push alerts require adding this app to your Home Screen on iOS 16.4+.
                </p>
                <button
                  type="button"
                  onClick={handleEnablePush}
                  disabled={isSubscribingPush}
                  style={{
                    width: "100%",
                    padding: "8px 12px",
                    background: "var(--accent-dim)",
                    color: "var(--accent)",
                    border: "1px solid var(--accent)",
                    borderRadius: "var(--radius-btn)",
                    fontSize: "13px",
                    fontWeight: 600,
                    gap: "6px",
                  }}
                >
                  <Bell size={14} />
                  {isSubscribingPush ? "Configuring..." : "Enable Push Alerts"}
                </button>
                {pushStatus && (
                  <div style={{ marginTop: "8px", fontSize: "11.5px", color: "var(--in)" }}>
                    {pushStatus}
                  </div>
                )}
              </div>
            </div>

            {/* Log Out */}
            <div style={{ paddingTop: "8px" }}>
              <button
                type="button"
                onClick={onLogOut}
                style={{
                  width: "100%",
                  padding: "11px",
                  background: "rgba(242, 114, 107, 0.12)",
                  color: "var(--out)",
                  border: "1px solid rgba(242, 114, 107, 0.3)",
                  borderRadius: "var(--radius-btn)",
                  fontWeight: 600,
                  fontSize: "14px",
                  gap: "6px",
                }}
              >
                <LogOut size={16} />
                Sign Out
              </button>
            </div>
          </div>
        )}

        {/* 2. Cards Section */}
        {activeSection === "cards" && (
          <div>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
              <span style={{ fontSize: "13px", color: "var(--muted)", fontWeight: 600 }}>Your Accounts ({cards.length})</span>
              <button
                onClick={() => setShowAddCard(!showAddCard)}
                style={{
                  background: "var(--accent)",
                  color: "#fff",
                  padding: "4px 10px",
                  borderRadius: "var(--radius-btn)",
                  fontSize: "12px",
                  fontWeight: 600,
                  gap: "4px",
                }}
              >
                <Plus size={14} />
                {showAddCard ? "Cancel" : "Add Card"}
              </button>
            </div>

            {showAddCard && (
              <form onSubmit={handleCreateCard} style={{ background: "var(--panel2)", padding: "12px", borderRadius: "var(--radius-btn)", marginBottom: "12px" }}>
                <div className="form-group">
                  <label className="form-label">Card / Account Name</label>
                  <input
                    type="text"
                    value={cardName}
                    onChange={(e) => setCardName(e.target.value)}
                    placeholder="e.g. Nayapay, HBL, Cash"
                    className="form-input"
                    required
                  />
                </div>
                <div className="form-group">
                  <label className="form-label">Opening Balance (PKR)</label>
                  <input
                    type="text"
                    inputMode="decimal"
                    value={openingBalanceRaw}
                    onChange={(e) => setOpeningBalanceRaw(e.target.value)}
                    className="form-input mono"
                    required
                  />
                </div>
                <div className="form-group">
                  <label className="form-label">Opening Date</label>
                  <input
                    type="date"
                    value={openingDate}
                    onChange={(e) => setOpeningDate(e.target.value)}
                    className="form-input"
                    required
                  />
                </div>
                <button
                  type="submit"
                  style={{
                    width: "100%",
                    padding: "10px",
                    background: "var(--accent)",
                    color: "#fff",
                    borderRadius: "var(--radius-btn)",
                    fontWeight: 600,
                  }}
                >
                  Create Account
                </button>
              </form>
            )}

            <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
              {cards.map((c) => (
                <div
                  key={c.id}
                  style={{
                    padding: "10px 12px",
                    background: "var(--panel2)",
                    borderRadius: "var(--radius-btn)",
                    border: "1px solid var(--border)",
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                  }}
                >
                  <div>
                    <div style={{ fontWeight: 600, fontSize: "14px" }}>
                      {c.name} {c.is_primary && <span style={{ color: "var(--star)" }}>★ Primary</span>}
                    </div>
                    <div className="mono" style={{ fontSize: "12px", color: "var(--muted)" }}>
                      Opening: {formatPkr(c.opening_balance)}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 3. Categories Section */}
        {activeSection === "categories" && (
          <div>
            <form onSubmit={handleCreateCategory} style={{ display: "flex", gap: "8px", marginBottom: "14px" }}>
              <input
                type="text"
                placeholder="New category name..."
                value={newCatName}
                onChange={(e) => setNewCatName(e.target.value)}
                className="form-input"
                style={{ flex: 1 }}
              />
              <button
                type="submit"
                style={{
                  background: "var(--accent)",
                  color: "#fff",
                  padding: "0 14px",
                  borderRadius: "var(--radius-btn)",
                  fontWeight: 600,
                  fontSize: "13px",
                }}
              >
                Add
              </button>
            </form>

            <div style={{ display: "flex", flexWrap: "wrap", gap: "6px" }}>
              {categories.map((cat) => (
                <span
                  key={cat.id}
                  className="badge badge-neutral"
                  style={{ padding: "6px 10px", fontSize: "12px" }}
                >
                  {cat.name} {cat.is_default && <span style={{ opacity: 0.5, marginLeft: "4px" }}>(default)</span>}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

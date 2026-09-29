import React, { useState } from "react";
import { Plus, Pause, Play, Trash2, X, RefreshCw, AlertCircle } from "lucide-react";
import { Card, Subscription } from "../types";
import {
  advanceSubscriptionDate,
  calcSubscriptionMonthlyCost,
  formatDate,
  formatPkr,
  generateUuid,
  parsePkrInput,
  todayIso,
  totalMonthlySubscriptionCost,
} from "../lib/utils";

interface SubscriptionsViewProps {
  subscriptions: Subscription[];
  cards: Card[];
  onAddSubscription: (sub: Subscription) => Promise<void>;
  onUpdateSubscription: (sub: Subscription) => Promise<void>;
  onDeleteSubscription: (subId: string) => Promise<void>;
  isAddModalOpen: boolean;
  setIsAddModalOpen: (open: boolean) => void;
}

export const SubscriptionsView: React.FC<SubscriptionsViewProps> = ({
  subscriptions,
  cards,
  onAddSubscription,
  onUpdateSubscription,
  onDeleteSubscription,
  isAddModalOpen,
  setIsAddModalOpen,
}) => {
  const [editingSub, setEditingSub] = useState<Subscription | null>(null);

  // Form states
  const [formName, setFormName] = useState("");
  const [formAmountRaw, setFormAmountRaw] = useState("");
  const [formCardId, setFormCardId] = useState(cards[0]?.id || "");
  const [formCycleType, setFormCycleType] = useState<"monthly" | "yearly" | "custom">("monthly");
  const [formCustomDays, setFormCustomDays] = useState("30");
  const [formStartDate, setFormStartDate] = useState(todayIso());
  const [formNextDueDate, setFormNextDueDate] = useState(todayIso());
  const [formError, setFormError] = useState<string | null>(null);

  const activeSubs = subscriptions.filter((s) => !s.deleted_at);
  const totalMonthly = totalMonthlySubscriptionCost(activeSubs);
  const pausedCount = activeSubs.filter((s) => s.is_paused).length;

  const openAdd = () => {
    setFormName("");
    setFormAmountRaw("");
    setFormCardId(cards[0]?.id || "");
    setFormCycleType("monthly");
    setFormCustomDays("30");
    setFormStartDate(todayIso());
    setFormNextDueDate(advanceSubscriptionDate(todayIso(), "monthly"));
    setFormError(null);
    setIsAddModalOpen(true);
  };

  const openEdit = (sub: Subscription) => {
    setEditingSub(sub);
    setFormName(sub.name);
    setFormAmountRaw(sub.amount.toString());
    setFormCardId(sub.card_id);

    const cycle = sub.billing_cycle.toLowerCase();
    if (cycle === "monthly") {
      setFormCycleType("monthly");
    } else if (cycle === "yearly") {
      setFormCycleType("yearly");
    } else if (cycle.startsWith("custom:")) {
      setFormCycleType("custom");
      setFormCustomDays(cycle.split(":")[1] || "30");
    } else {
      setFormCycleType("monthly");
    }

    setFormStartDate(sub.start_date || sub.next_due_date);
    setFormNextDueDate(sub.next_due_date);
    setFormError(null);
  };

  const handleCycleChange = (type: "monthly" | "yearly" | "custom", customDaysVal = formCustomDays) => {
    setFormCycleType(type);
    const cycleStr = type === "custom" ? `custom:${customDaysVal}` : type;
    setFormNextDueDate(advanceSubscriptionDate(formStartDate, cycleStr));
  };

  const handleStartDateChange = (date: string) => {
    setFormStartDate(date);
    const cycleStr = formCycleType === "custom" ? `custom:${formCustomDays}` : formCycleType;
    setFormNextDueDate(advanceSubscriptionDate(date, cycleStr));
  };

  const handleSaveAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    const amount = parsePkrInput(formAmountRaw);
    if (!amount || amount <= 0) {
      setFormError("Please enter a valid amount.");
      return;
    }
    if (!formName.trim()) {
      setFormError("Please enter a subscription name.");
      return;
    }

    const cycle = formCycleType === "custom" ? `custom:${parseInt(formCustomDays, 10) || 30}` : formCycleType;

    const newSub: Subscription = {
      id: generateUuid(),
      card_id: formCardId || cards[0]?.id || "",
      name: formName.trim(),
      amount,
      billing_cycle: cycle,
      start_date: formStartDate,
      next_due_date: formNextDueDate,
      is_paused: false,
      updated_at: new Date().toISOString(),
      deleted_at: null,
    };

    await onAddSubscription(newSub);
    setIsAddModalOpen(false);
  };

  const handleSaveEdit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingSub) return;

    const amount = parsePkrInput(formAmountRaw);
    if (!amount || amount <= 0) {
      setFormError("Please enter a valid amount.");
      return;
    }
    if (!formName.trim()) {
      setFormError("Please enter a subscription name.");
      return;
    }

    const cycle = formCycleType === "custom" ? `custom:${parseInt(formCustomDays, 10) || 30}` : formCycleType;

    const updated: Subscription = {
      ...editingSub,
      card_id: formCardId,
      name: formName.trim(),
      amount,
      billing_cycle: cycle,
      start_date: formStartDate,
      next_due_date: formNextDueDate,
      updated_at: new Date().toISOString(),
    };

    await onUpdateSubscription(updated);
    setEditingSub(null);
  };

  const togglePause = async (sub: Subscription, e: React.MouseEvent) => {
    e.stopPropagation();
    const updated: Subscription = {
      ...sub,
      is_paused: !sub.is_paused,
      updated_at: new Date().toISOString(),
    };
    await onUpdateSubscription(updated);
  };

  const handleDelete = async () => {
    if (!editingSub) return;
    if (confirm(`Delete subscription "${editingSub.name}"?`)) {
      await onDeleteSubscription(editingSub.id);
      setEditingSub(null);
    }
  };

  const formatCycleBadge = (cycleStr: string) => {
    const c = cycleStr.toLowerCase();
    if (c === "monthly") return "Monthly";
    if (c === "yearly") return "Yearly";
    if (c.startsWith("custom:")) return `Every ${c.split(":")[1]} Days`;
    return cycleStr;
  };

  return (
    <div style={{ paddingBottom: "100px" }}>
      {/* 1. Monthly Cost Prorated Summary Card */}
      <div
        className="card"
        style={{
          margin: "16px",
          background: "linear-gradient(145deg, var(--panel) 0%, var(--panel2) 100%)",
          border: "1px solid var(--border)",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
          <div>
            <div style={{ fontSize: "12px", color: "var(--muted)", textTransform: "uppercase", letterSpacing: "0.05em" }}>
              Total Prorated Monthly Cost
            </div>
            <div className="mono" style={{ fontSize: "28px", fontWeight: 700, color: "var(--out)", marginTop: "4px" }}>
              {formatPkr(totalMonthly)}
              <span style={{ fontSize: "14px", color: "var(--muted)", fontWeight: 400 }}> / mo</span>
            </div>
          </div>
          <button
            onClick={openAdd}
            style={{
              background: "var(--accent)",
              color: "#ffffff",
              padding: "6px 12px",
              borderRadius: "var(--radius-btn)",
              fontSize: "12px",
              fontWeight: 600,
              gap: "4px",
            }}
          >
            <Plus size={15} />
            New
          </button>
        </div>

        <div style={{ display: "flex", gap: "12px", marginTop: "14px", paddingTop: "12px", borderTop: "1px solid var(--border)", fontSize: "12px", color: "var(--muted)" }}>
          <div>
            Active: <strong style={{ color: "var(--text)" }}>{activeSubs.length - pausedCount}</strong>
          </div>
          {pausedCount > 0 && (
            <div>
              Paused: <strong style={{ color: "var(--star)" }}>{pausedCount}</strong>
            </div>
          )}
        </div>
      </div>

      {/* 2. Subscriptions Cards List */}
      <div style={{ padding: "0 16px" }}>
        <div style={{ fontSize: "13px", fontWeight: 600, color: "var(--muted)", marginBottom: "8px", textTransform: "uppercase", letterSpacing: "0.04em" }}>
          Active Subscriptions ({activeSubs.length})
        </div>

        {activeSubs.length === 0 ? (
          <div
            style={{
              padding: "40px 20px",
              textAlign: "center",
              background: "var(--panel)",
              border: "1px dashed var(--border)",
              borderRadius: "var(--radius-card)",
              color: "var(--muted)",
            }}
          >
            No recurring subscriptions yet. Tap <strong>+ New</strong> to add one.
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
            {activeSubs.map((sub) => {
              const cardName = cards.find((c) => c.id === sub.card_id)?.name || "Default Card";
              const prorated = calcSubscriptionMonthlyCost(sub);
              const isDueSoon = sub.next_due_date <= todayIso();

              return (
                <div
                  key={sub.id}
                  onClick={() => openEdit(sub)}
                  style={{
                    background: "var(--panel)",
                    border: `1px solid ${sub.is_paused ? "var(--border)" : isDueSoon ? "rgba(242, 114, 107, 0.4)" : "var(--border)"}`,
                    borderRadius: "var(--radius-card)",
                    padding: "14px 16px",
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    cursor: "pointer",
                    opacity: sub.is_paused ? 0.6 : 1,
                  }}
                >
                  <div style={{ maxWidth: "60%" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "4px" }}>
                      <span style={{ fontWeight: 600, fontSize: "15px", color: "var(--text)" }}>
                        {sub.name}
                      </span>
                      <span className="badge badge-neutral" style={{ fontSize: "10px" }}>
                        {formatCycleBadge(sub.billing_cycle)}
                      </span>
                      {sub.is_paused && (
                        <span className="badge badge-gold" style={{ fontSize: "10px" }}>
                          Paused
                        </span>
                      )}
                    </div>

                    <div style={{ fontSize: "12px", color: "var(--muted)" }}>
                      Charged to: <span style={{ color: "var(--text)" }}>{cardName}</span>
                    </div>

                    <div style={{ fontSize: "11.5px", color: isDueSoon && !sub.is_paused ? "var(--out)" : "var(--muted)", marginTop: "2px", display: "flex", alignItems: "center", gap: "4px" }}>
                      {isDueSoon && !sub.is_paused && <AlertCircle size={12} />}
                      Next Due: {formatDate(sub.next_due_date)}
                    </div>
                  </div>

                  <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
                    <div style={{ textAlign: "right" }}>
                      <div className="mono" style={{ fontSize: "16px", fontWeight: 700, color: "var(--text)" }}>
                        {formatPkr(sub.amount)}
                      </div>
                      {sub.billing_cycle.toLowerCase() !== "monthly" && (
                        <div className="mono" style={{ fontSize: "11px", color: "var(--muted)" }}>
                          ({formatPkr(prorated)}/mo)
                        </div>
                      )}
                    </div>

                    <button
                      type="button"
                      onClick={(e) => togglePause(sub, e)}
                      title={sub.is_paused ? "Resume" : "Pause"}
                      style={{
                        background: sub.is_paused ? "var(--in-btn-bg)" : "var(--panel2)",
                        color: sub.is_paused ? "var(--in)" : "var(--muted)",
                        padding: "8px",
                        borderRadius: "50%",
                        border: "1px solid var(--border)",
                      }}
                    >
                      {sub.is_paused ? <Play size={14} /> : <Pause size={14} />}
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* 3. Add Modal */}
      {isAddModalOpen && (
        <div className="modal-overlay" onClick={() => setIsAddModalOpen(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-handle" />
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
              <h3 style={{ margin: 0, fontSize: "18px" }}>Add Subscription</h3>
              <button
                onClick={() => setIsAddModalOpen(false)}
                style={{ background: "transparent", color: "var(--muted)", padding: "4px" }}
              >
                <X size={18} />
              </button>
            </div>

            {formError && (
              <div style={{ background: "rgba(242, 114, 107, 0.15)", color: "var(--out)", padding: "8px 12px", borderRadius: "6px", fontSize: "13px", marginBottom: "14px" }}>
                {formError}
              </div>
            )}

            <form onSubmit={handleSaveAdd}>
              <div className="form-group">
                <label className="form-label">Subscription Name</label>
                <input
                  type="text"
                  placeholder="e.g. Netflix, Spotify, Gym"
                  value={formName}
                  onChange={(e) => setFormName(e.target.value)}
                  className="form-input"
                  autoFocus
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Amount (PKR)</label>
                <input
                  type="text"
                  inputMode="decimal"
                  placeholder="e.g. 1,100"
                  value={formAmountRaw}
                  onChange={(e) => setFormAmountRaw(e.target.value)}
                  className="form-input mono"
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Billing Cycle</label>
                <div style={{ display: "flex", gap: "6px" }}>
                  {(["monthly", "yearly", "custom"] as const).map((cycle) => (
                    <button
                      key={cycle}
                      type="button"
                      onClick={() => handleCycleChange(cycle)}
                      style={{
                        flex: 1,
                        padding: "8px",
                        fontSize: "12px",
                        fontWeight: 600,
                        borderRadius: "var(--radius-btn)",
                        background: formCycleType === cycle ? "var(--accent)" : "var(--panel2)",
                        color: formCycleType === cycle ? "#ffffff" : "var(--muted)",
                        border: `1px solid ${formCycleType === cycle ? "var(--accent)" : "var(--border)"}`,
                      }}
                    >
                      {cycle === "monthly" ? "Monthly" : cycle === "yearly" ? "Yearly" : "Custom Days"}
                    </button>
                  ))}
                </div>
              </div>

              {formCycleType === "custom" && (
                <div className="form-group">
                  <label className="form-label">Repeat Every (Days)</label>
                  <input
                    type="number"
                    min="1"
                    value={formCustomDays}
                    onChange={(e) => {
                      setFormCustomDays(e.target.value);
                      handleCycleChange("custom", e.target.value);
                    }}
                    className="form-input mono"
                    required
                  />
                </div>
              )}

              <div className="form-group">
                <label className="form-label">Charging Account / Card</label>
                <select
                  value={formCardId}
                  onChange={(e) => setFormCardId(e.target.value)}
                  className="form-input"
                >
                  {cards.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name} {c.is_primary ? "★" : ""}
                    </option>
                  ))}
                </select>
              </div>

              <div style={{ display: "flex", gap: "8px" }}>
                <div className="form-group" style={{ flex: 1 }}>
                  <label className="form-label">Start Date</label>
                  <input
                    type="date"
                    value={formStartDate}
                    onChange={(e) => handleStartDateChange(e.target.value)}
                    className="form-input"
                    required
                  />
                </div>
                <div className="form-group" style={{ flex: 1 }}>
                  <label className="form-label">Next Due Date</label>
                  <input
                    type="date"
                    value={formNextDueDate}
                    onChange={(e) => setFormNextDueDate(e.target.value)}
                    className="form-input"
                    required
                  />
                </div>
              </div>

              <button
                type="submit"
                style={{
                  width: "100%",
                  padding: "12px",
                  background: "var(--accent)",
                  color: "#ffffff",
                  fontSize: "15px",
                  fontWeight: 600,
                  borderRadius: "var(--radius-btn)",
                  marginTop: "8px",
                }}
              >
                Add Subscription
              </button>
            </form>
          </div>
        </div>
      )}

      {/* 4. Edit Modal */}
      {editingSub && (
        <div className="modal-overlay" onClick={() => setEditingSub(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-handle" />
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
              <h3 style={{ margin: 0, fontSize: "18px" }}>Edit Subscription</h3>
              <button
                onClick={() => setEditingSub(null)}
                style={{ background: "transparent", color: "var(--muted)", padding: "4px" }}
              >
                <X size={18} />
              </button>
            </div>

            {formError && (
              <div style={{ background: "rgba(242, 114, 107, 0.15)", color: "var(--out)", padding: "8px 12px", borderRadius: "6px", fontSize: "13px", marginBottom: "14px" }}>
                {formError}
              </div>
            )}

            <form onSubmit={handleSaveEdit}>
              <div className="form-group">
                <label className="form-label">Subscription Name</label>
                <input
                  type="text"
                  value={formName}
                  onChange={(e) => setFormName(e.target.value)}
                  className="form-input"
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Amount (PKR)</label>
                <input
                  type="text"
                  inputMode="decimal"
                  value={formAmountRaw}
                  onChange={(e) => setFormAmountRaw(e.target.value)}
                  className="form-input mono"
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Billing Cycle</label>
                <div style={{ display: "flex", gap: "6px" }}>
                  {(["monthly", "yearly", "custom"] as const).map((cycle) => (
                    <button
                      key={cycle}
                      type="button"
                      onClick={() => handleCycleChange(cycle)}
                      style={{
                        flex: 1,
                        padding: "8px",
                        fontSize: "12px",
                        fontWeight: 600,
                        borderRadius: "var(--radius-btn)",
                        background: formCycleType === cycle ? "var(--accent)" : "var(--panel2)",
                        color: formCycleType === cycle ? "#ffffff" : "var(--muted)",
                        border: `1px solid ${formCycleType === cycle ? "var(--accent)" : "var(--border)"}`,
                      }}
                    >
                      {cycle === "monthly" ? "Monthly" : cycle === "yearly" ? "Yearly" : "Custom Days"}
                    </button>
                  ))}
                </div>
              </div>

              {formCycleType === "custom" && (
                <div className="form-group">
                  <label className="form-label">Repeat Every (Days)</label>
                  <input
                    type="number"
                    min="1"
                    value={formCustomDays}
                    onChange={(e) => {
                      setFormCustomDays(e.target.value);
                      handleCycleChange("custom", e.target.value);
                    }}
                    className="form-input mono"
                    required
                  />
                </div>
              )}

              <div className="form-group">
                <label className="form-label">Charging Account / Card</label>
                <select
                  value={formCardId}
                  onChange={(e) => setFormCardId(e.target.value)}
                  className="form-input"
                >
                  {cards.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name} {c.is_primary ? "★" : ""}
                    </option>
                  ))}
                </select>
              </div>

              <div style={{ display: "flex", gap: "8px" }}>
                <div className="form-group" style={{ flex: 1 }}>
                  <label className="form-label">Start Date</label>
                  <input
                    type="date"
                    value={formStartDate}
                    onChange={(e) => handleStartDateChange(e.target.value)}
                    className="form-input"
                    required
                  />
                </div>
                <div className="form-group" style={{ flex: 1 }}>
                  <label className="form-label">Next Due Date</label>
                  <input
                    type="date"
                    value={formNextDueDate}
                    onChange={(e) => setFormNextDueDate(e.target.value)}
                    className="form-input"
                    required
                  />
                </div>
              </div>

              <div style={{ display: "flex", gap: "10px", marginTop: "14px" }}>
                <button
                  type="button"
                  onClick={handleDelete}
                  style={{
                    background: "rgba(242, 114, 107, 0.12)",
                    color: "var(--out)",
                    border: "1px solid rgba(242, 114, 107, 0.3)",
                    padding: "12px 16px",
                    borderRadius: "var(--radius-btn)",
                    gap: "6px",
                  }}
                >
                  <Trash2 size={16} />
                  Delete
                </button>

                <button
                  type="submit"
                  style={{
                    flex: 1,
                    padding: "12px",
                    background: "var(--accent)",
                    color: "#ffffff",
                    fontSize: "15px",
                    fontWeight: 600,
                    borderRadius: "var(--radius-btn)",
                  }}
                >
                  Save Changes
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};

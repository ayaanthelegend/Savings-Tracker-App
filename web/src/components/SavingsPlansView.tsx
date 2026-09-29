import React, { useState } from "react";
import { Plus, Target, CheckCircle2, AlertTriangle, ChevronRight, X, Trash2, Calendar, ShieldCheck } from "lucide-react";
import { Card, SavingsPlan, Subscription, Transaction } from "../types";
import {
  computePlanProgress,
  formatDate,
  formatPkr,
  formatPkrWhole,
  generateUuid,
  parsePkrInput,
  todayIso,
  totalMonthlySubscriptionCost,
} from "../lib/utils";

interface SavingsPlansViewProps {
  plans: SavingsPlan[];
  cards: Card[];
  transactions: Transaction[];
  subscriptions: Subscription[];
  onAddPlan: (plan: SavingsPlan) => Promise<void>;
  onUpdatePlan: (plan: SavingsPlan) => Promise<void>;
  onDeletePlan: (planId: string) => Promise<void>;
  isAddModalOpen: boolean;
  setIsAddModalOpen: (open: boolean) => void;
}

export const SavingsPlansView: React.FC<SavingsPlansViewProps> = ({
  plans,
  cards,
  transactions,
  subscriptions,
  onAddPlan,
  onUpdatePlan,
  onDeletePlan,
  isAddModalOpen,
  setIsAddModalOpen,
}) => {
  const [subTab, setSubTab] = useState<"active" | "history">("active");
  const [globalDeductOverspending, setGlobalDeductOverspending] = useState(true);

  // Modals
  const [selectedPlanDetail, setSelectedPlanDetail] = useState<SavingsPlan | null>(null);
  const [editingPlan, setEditingPlan] = useState<SavingsPlan | null>(null);

  // Form states
  const [formName, setFormName] = useState("");
  const [formTargetRaw, setFormTargetRaw] = useState("");
  const [formDeadline, setFormDeadline] = useState("");
  const [formIncomeRaw, setFormIncomeRaw] = useState("");
  const [formLimitOverrideRaw, setFormLimitOverrideRaw] = useState("");
  const [formLinkedCards, setFormLinkedCards] = useState<string[]>([]);
  const [formDeductOverspending, setFormDeductOverspending] = useState(true);
  const [formError, setFormError] = useState<string | null>(null);

  const activePlans = plans.filter((p) => p.is_active && !p.deleted_at);
  const historyPlans = plans.filter((p) => !p.is_active && !p.deleted_at);

  const displayedPlans = subTab === "active" ? activePlans : historyPlans;

  const openAdd = () => {
    setFormName("");
    setFormTargetRaw("");
    // Default deadline: 6 months from today
    const d = new Date();
    d.setMonth(d.getMonth() + 6);
    setFormDeadline(d.toISOString().split("T")[0]);
    setFormIncomeRaw("150,000");
    setFormLimitOverrideRaw("");
    setFormLinkedCards([]);
    setFormDeductOverspending(globalDeductOverspending);
    setFormError(null);
    setIsAddModalOpen(true);
  };

  const openEdit = (plan: SavingsPlan) => {
    setEditingPlan(plan);
    setFormName(plan.name);
    setFormTargetRaw(plan.target_amount.toString());
    setFormDeadline(plan.deadline);
    setFormIncomeRaw(plan.monthly_income.toString());
    setFormLimitOverrideRaw(plan.spending_limit != null ? plan.spending_limit.toString() : "");
    setFormLinkedCards(plan.linked_card_ids || []);
    setFormDeductOverspending(plan.deduct_overspending);
    setFormError(null);
    setSelectedPlanDetail(null);
  };

  const handleSaveAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    const target = parsePkrInput(formTargetRaw);
    const income = parsePkrInput(formIncomeRaw);
    const limitOverride = formLimitOverrideRaw.trim() ? parsePkrInput(formLimitOverrideRaw) : null;

    if (!target || target <= 0) {
      setFormError("Please enter a valid target amount.");
      return;
    }
    if (!income || income <= 0) {
      setFormError("Please enter your estimated monthly income.");
      return;
    }
    if (!formDeadline) {
      setFormError("Please select a target deadline.");
      return;
    }

    const newPlan: SavingsPlan = {
      id: generateUuid(),
      name: formName.trim(),
      target_amount: target,
      deadline: formDeadline,
      monthly_income: income,
      spending_limit: limitOverride,
      is_active: true,
      deduct_overspending: formDeductOverspending,
      linked_card_ids: formLinkedCards,
      created_at: todayIso(),
      updated_at: new Date().toISOString(),
      deleted_at: null,
    };

    await onAddPlan(newPlan);
    setIsAddModalOpen(false);
  };

  const handleSaveEdit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingPlan) return;

    const target = parsePkrInput(formTargetRaw);
    const income = parsePkrInput(formIncomeRaw);
    const limitOverride = formLimitOverrideRaw.trim() ? parsePkrInput(formLimitOverrideRaw) : null;

    if (!target || target <= 0) {
      setFormError("Please enter a valid target amount.");
      return;
    }
    if (!income || income <= 0) {
      setFormError("Please enter your monthly income.");
      return;
    }

    const updated: SavingsPlan = {
      ...editingPlan,
      name: formName.trim(),
      target_amount: target,
      deadline: formDeadline,
      monthly_income: income,
      spending_limit: limitOverride,
      deduct_overspending: formDeductOverspending,
      linked_card_ids: formLinkedCards,
      updated_at: new Date().toISOString(),
    };

    await onUpdatePlan(updated);
    setEditingPlan(null);
  };

  const handleDelete = async () => {
    if (!editingPlan) return;
    if (confirm(`Delete savings plan "${editingPlan.name}"?`)) {
      await onDeletePlan(editingPlan.id);
      setEditingPlan(null);
    }
  };

  const toggleLinkedCard = (cardId: string) => {
    if (formLinkedCards.includes(cardId)) {
      setFormLinkedCards(formLinkedCards.filter((id) => id !== cardId));
    } else {
      setFormLinkedCards([...formLinkedCards, cardId]);
    }
  };

  return (
    <div style={{ paddingBottom: "100px" }}>
      {/* 1. Header Controls: Sub-tabs & Global Deduct Toggle */}
      <div style={{ padding: "16px 16px 8px 16px", display: "flex", flexDirection: "column", gap: "12px" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          {/* Active / History Switcher */}
          <div style={{ display: "flex", background: "var(--panel)", borderRadius: "var(--radius-tab)", padding: "3px", border: "1px solid var(--border)" }}>
            <button
              onClick={() => setSubTab("active")}
              style={{
                padding: "6px 14px",
                fontSize: "13px",
                fontWeight: 600,
                borderRadius: "var(--radius-btn)",
                background: subTab === "active" ? "var(--accent)" : "transparent",
                color: subTab === "active" ? "#ffffff" : "var(--muted)",
              }}
            >
              Active ({activePlans.length})
            </button>
            <button
              onClick={() => setSubTab("history")}
              style={{
                padding: "6px 14px",
                fontSize: "13px",
                fontWeight: 600,
                borderRadius: "var(--radius-btn)",
                background: subTab === "history" ? "var(--accent)" : "transparent",
                color: subTab === "history" ? "#ffffff" : "var(--muted)",
              }}
            >
              History ({historyPlans.length})
            </button>
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
            New Goal
          </button>
        </div>

        {/* Global Deduct Overspending setting */}
        <div
          style={{
            background: "var(--panel2)",
            border: "1px solid var(--border)",
            borderRadius: "var(--radius-btn)",
            padding: "8px 12px",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <span style={{ fontSize: "12px", color: "var(--muted)" }}>
            Deduct overspending from surplus
          </span>
          <input
            type="checkbox"
            checked={globalDeductOverspending}
            onChange={(e) => setGlobalDeductOverspending(e.target.checked)}
            style={{ width: "16px", height: "16px", accentColor: "var(--accent)", cursor: "pointer" }}
          />
        </div>
      </div>

      {/* 2. Savings Plans List */}
      <div style={{ padding: "0 16px", display: "flex", flexDirection: "column", gap: "14px" }}>
        {displayedPlans.length === 0 ? (
          <div
            style={{
              padding: "40px 20px",
              textAlign: "center",
              background: "var(--panel)",
              border: "1px dashed var(--border)",
              borderRadius: "var(--radius-card)",
              color: "var(--muted)",
              marginTop: "8px",
            }}
          >
            {subTab === "active"
              ? "No active savings goals. Tap + New Goal to create one."
              : "No completed or past savings plans yet."}
          </div>
        ) : (
          displayedPlans.map((plan) => {
            const proratedSubCost = totalMonthlySubscriptionCost(
              subscriptions,
              plan.linked_card_ids.length > 0 ? plan.linked_card_ids : undefined
            );

            const calc = computePlanProgress(
              plan,
              transactions,
              proratedSubCost,
              todayIso(),
              plan.deduct_overspending ?? globalDeductOverspending
            );

            const pct = Math.min(100, Math.round(calc.percent_reached));

            return (
              <div
                key={plan.id}
                className="card"
                onClick={() => setSelectedPlanDetail(plan)}
                style={{
                  cursor: "pointer",
                  transition: "border-color 0.15s",
                  border: `1px solid ${calc.is_feasible ? "var(--border)" : "rgba(242, 114, 107, 0.4)"}`,
                }}
              >
                {/* Header: Title & Feasibility Badge */}
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "10px" }}>
                  <div>
                    <h3 style={{ fontSize: "16px", fontWeight: 700, margin: "0 0 2px 0" }}>
                      {plan.name}
                    </h3>
                    <div style={{ fontSize: "12px", color: "var(--muted)" }}>
                      Deadline: {formatDate(plan.deadline)} ({calc.days_remaining > 0 ? `${calc.days_remaining}d left` : "Expired"})
                    </div>
                  </div>

                  {calc.is_feasible ? (
                    <span className="badge badge-in" style={{ fontSize: "10.5px" }}>
                      Feasible
                    </span>
                  ) : (
                    <span className="badge badge-out" style={{ fontSize: "10.5px" }}>
                      Shortage: {formatPkrWhole(calc.shortage)}
                    </span>
                  )}
                </div>

                {/* Amount and Progress */}
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: "6px" }}>
                  <div className="mono" style={{ fontSize: "20px", fontWeight: 700, color: "var(--in)" }}>
                    {formatPkr(calc.total_saved)}
                  </div>
                  <div className="mono" style={{ fontSize: "13px", color: "var(--muted)" }}>
                    Target: {formatPkr(plan.target_amount)} ({pct}%)
                  </div>
                </div>

                {/* Progress Bar */}
                <div
                  style={{
                    height: "8px",
                    background: "var(--panel2)",
                    borderRadius: "var(--radius-bar)",
                    overflow: "hidden",
                    marginBottom: "12px",
                  }}
                >
                  <div
                    style={{
                      height: "100%",
                      width: `${pct}%`,
                      background: pct >= 100 ? "var(--star)" : "var(--in)",
                      borderRadius: "var(--radius-bar)",
                      transition: "width 0.4s ease-out",
                    }}
                  />
                </div>

                {/* Footer Metrics */}
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    paddingTop: "10px",
                    borderTop: "1px solid var(--border)",
                    fontSize: "12px",
                    color: "var(--muted)",
                  }}
                >
                  <div>
                    Monthly Limit:{" "}
                    <span className="mono" style={{ color: "var(--text)", fontWeight: 600 }}>
                      {formatPkrWhole(calc.effective_limit)}
                    </span>
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: "2px", color: "var(--accent)" }}>
                    Details <ChevronRight size={14} />
                  </div>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* 3. Detail Drawer / Modal for Monthly Breakdowns */}
      {selectedPlanDetail && (
        <div className="modal-overlay" onClick={() => setSelectedPlanDetail(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-handle" />
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
              <div>
                <h3 style={{ margin: 0, fontSize: "18px" }}>{selectedPlanDetail.name}</h3>
                <div style={{ fontSize: "12px", color: "var(--muted)" }}>
                  Target: {formatPkr(selectedPlanDetail.target_amount)} • Deadline: {formatDate(selectedPlanDetail.deadline)}
                </div>
              </div>
              <button
                onClick={() => setSelectedPlanDetail(null)}
                style={{ background: "transparent", color: "var(--muted)", padding: "4px" }}
              >
                <X size={18} />
              </button>
            </div>

            {/* Calculations Summary */}
            {(() => {
              const prorated = totalMonthlySubscriptionCost(
                subscriptions,
                selectedPlanDetail.linked_card_ids.length > 0 ? selectedPlanDetail.linked_card_ids : undefined
              );
              const calc = computePlanProgress(
                selectedPlanDetail,
                transactions,
                prorated,
                todayIso(),
                selectedPlanDetail.deduct_overspending ?? globalDeductOverspending
              );

              return (
                <div>
                  <div style={{ background: "var(--panel2)", borderRadius: "var(--radius-card)", padding: "14px", marginBottom: "16px", border: "1px solid var(--border)" }}>
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px", fontSize: "12.5px" }}>
                      <div>
                        <div style={{ color: "var(--muted)" }}>Required / Month</div>
                        <div className="mono" style={{ fontWeight: 600, color: "var(--text)" }}>
                          {formatPkrWhole(calc.required_savings_per_month)}
                        </div>
                      </div>
                      <div>
                        <div style={{ color: "var(--muted)" }}>Spending Limit</div>
                        <div className="mono" style={{ fontWeight: 600, color: "var(--text)" }}>
                          {formatPkrWhole(calc.effective_limit)}
                        </div>
                      </div>
                      <div>
                        <div style={{ color: "var(--muted)" }}>Gross Saved</div>
                        <div className="mono" style={{ fontWeight: 600, color: "var(--in)" }}>
                          {formatPkr(calc.gross_surplus)}
                        </div>
                      </div>
                      <div>
                        <div style={{ color: "var(--muted)" }}>Net Saved</div>
                        <div className="mono" style={{ fontWeight: 600, color: "var(--in)" }}>
                          {formatPkr(calc.total_saved)}
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Month-by-month breakdown table */}
                  <h4 style={{ fontSize: "14px", marginBottom: "8px", textTransform: "uppercase", color: "var(--muted)" }}>
                    Monthly Surplus Breakdown
                  </h4>

                  <div style={{ display: "flex", flexDirection: "column", gap: "8px", maxHeight: "240px", overflowY: "auto" }}>
                    {calc.breakdowns.map((b) => (
                      <div
                        key={b.label}
                        style={{
                          padding: "10px 12px",
                          background: "var(--panel2)",
                          border: `1px solid ${b.is_overspent ? "rgba(242, 114, 107, 0.4)" : "var(--border)"}`,
                          borderRadius: "var(--radius-btn)",
                          display: "flex",
                          justifyContent: "space-between",
                          alignItems: "center",
                        }}
                      >
                        <div>
                          <div style={{ fontWeight: 600, fontSize: "13px" }}>
                            {b.label} {b.is_current_month && <span style={{ color: "var(--accent)" }}>(Current)</span>}
                          </div>
                          <div className="mono" style={{ fontSize: "11px", color: "var(--muted)" }}>
                            Spent: {formatPkr(b.actual_spent)} / Limit: {formatPkrWhole(b.spending_limit)}
                          </div>
                        </div>

                        <div style={{ textAlign: "right" }}>
                          <div
                            className="mono"
                            style={{
                              fontSize: "13px",
                              fontWeight: 700,
                              color: b.surplus >= 0 ? "var(--in)" : "var(--out)",
                            }}
                          >
                            {b.surplus >= 0 ? "+" : ""}
                            {formatPkr(b.surplus)}
                          </div>
                          {b.is_overspent && (
                            <span className="badge badge-out" style={{ fontSize: "9px" }}>
                              Overspent
                            </span>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>

                  <div style={{ display: "flex", gap: "10px", marginTop: "16px" }}>
                    <button
                      type="button"
                      onClick={() => openEdit(selectedPlanDetail)}
                      style={{
                        flex: 1,
                        padding: "12px",
                        background: "var(--accent)",
                        color: "#ffffff",
                        fontSize: "14px",
                        fontWeight: 600,
                        borderRadius: "var(--radius-btn)",
                      }}
                    >
                      Edit Goal Settings
                    </button>
                  </div>
                </div>
              );
            })()}
          </div>
        </div>
      )}

      {/* 4. Add Plan Modal */}
      {isAddModalOpen && (
        <div className="modal-overlay" onClick={() => setIsAddModalOpen(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-handle" />
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
              <h3 style={{ margin: 0, fontSize: "18px" }}>Create Savings Goal</h3>
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
                <label className="form-label">Goal Name</label>
                <input
                  type="text"
                  placeholder="e.g. New Laptop, Emergency Fund"
                  value={formName}
                  onChange={(e) => setFormName(e.target.value)}
                  className="form-input"
                  autoFocus
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Target Amount (PKR)</label>
                <input
                  type="text"
                  inputMode="decimal"
                  placeholder="e.g. 200,000"
                  value={formTargetRaw}
                  onChange={(e) => setFormTargetRaw(e.target.value)}
                  className="form-input mono"
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Target Deadline</label>
                <input
                  type="date"
                  value={formDeadline}
                  onChange={(e) => setFormDeadline(e.target.value)}
                  className="form-input"
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Monthly Income (PKR)</label>
                <input
                  type="text"
                  inputMode="decimal"
                  placeholder="e.g. 150,000"
                  value={formIncomeRaw}
                  onChange={(e) => setFormIncomeRaw(e.target.value)}
                  className="form-input mono"
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Spending Limit Override (Optional)</label>
                <input
                  type="text"
                  inputMode="decimal"
                  placeholder="Auto-calculated if left blank"
                  value={formLimitOverrideRaw}
                  onChange={(e) => setFormLimitOverrideRaw(e.target.value)}
                  className="form-input mono"
                />
              </div>

              {/* Linked Cards Selector */}
              <div className="form-group">
                <label className="form-label">Linked Accounts (blank = all cards)</label>
                <div style={{ display: "flex", flexWrap: "wrap", gap: "6px" }}>
                  {cards.map((c) => {
                    const isSelected = formLinkedCards.includes(c.id);
                    return (
                      <button
                        key={c.id}
                        type="button"
                        onClick={() => toggleLinkedCard(c.id)}
                        style={{
                          padding: "6px 10px",
                          fontSize: "12px",
                          borderRadius: "var(--radius-badge)",
                          background: isSelected ? "var(--accent)" : "var(--panel2)",
                          color: isSelected ? "#fff" : "var(--muted)",
                          border: `1px solid ${isSelected ? "var(--accent)" : "var(--border)"}`,
                        }}
                      >
                        {c.name}
                      </button>
                    );
                  })}
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
                Create Goal
              </button>
            </form>
          </div>
        </div>
      )}

      {/* 5. Edit Plan Modal */}
      {editingPlan && (
        <div className="modal-overlay" onClick={() => setEditingPlan(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-handle" />
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
              <h3 style={{ margin: 0, fontSize: "18px" }}>Edit Savings Goal</h3>
              <button
                onClick={() => setEditingPlan(null)}
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
                <label className="form-label">Goal Name</label>
                <input
                  type="text"
                  value={formName}
                  onChange={(e) => setFormName(e.target.value)}
                  className="form-input"
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Target Amount (PKR)</label>
                <input
                  type="text"
                  inputMode="decimal"
                  value={formTargetRaw}
                  onChange={(e) => setFormTargetRaw(e.target.value)}
                  className="form-input mono"
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Target Deadline</label>
                <input
                  type="date"
                  value={formDeadline}
                  onChange={(e) => setFormDeadline(e.target.value)}
                  className="form-input"
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Monthly Income (PKR)</label>
                <input
                  type="text"
                  inputMode="decimal"
                  value={formIncomeRaw}
                  onChange={(e) => setFormIncomeRaw(e.target.value)}
                  className="form-input mono"
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Spending Limit Override</label>
                <input
                  type="text"
                  inputMode="decimal"
                  placeholder="Auto-calculated if left blank"
                  value={formLimitOverrideRaw}
                  onChange={(e) => setFormLimitOverrideRaw(e.target.value)}
                  className="form-input mono"
                />
              </div>

              <div className="form-group">
                <label className="form-label">Linked Accounts</label>
                <div style={{ display: "flex", flexWrap: "wrap", gap: "6px" }}>
                  {cards.map((c) => {
                    const isSelected = formLinkedCards.includes(c.id);
                    return (
                      <button
                        key={c.id}
                        type="button"
                        onClick={() => toggleLinkedCard(c.id)}
                        style={{
                          padding: "6px 10px",
                          fontSize: "12px",
                          borderRadius: "var(--radius-badge)",
                          background: isSelected ? "var(--accent)" : "var(--panel2)",
                          color: isSelected ? "#fff" : "var(--muted)",
                          border: `1px solid ${isSelected ? "var(--accent)" : "var(--border)"}`,
                        }}
                      >
                        {c.name}
                      </button>
                    );
                  })}
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

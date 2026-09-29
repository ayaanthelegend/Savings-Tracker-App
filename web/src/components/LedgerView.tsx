import React, { useState, useMemo } from "react";
import { Search, Plus, Filter, ArrowUpRight, ArrowDownLeft, X, Trash2 } from "lucide-react";
import { Card, Category, Transaction } from "../types";
import { formatDate, formatPkr, formatPkrWhole, generateUuid, parsePkrInput, todayIso } from "../lib/utils";

interface LedgerViewProps {
  card: Card;
  transactions: Transaction[];
  categories: Category[];
  onAddTransaction: (tx: Transaction) => Promise<void>;
  onUpdateTransaction: (tx: Transaction) => Promise<void>;
  onDeleteTransaction: (txId: string) => Promise<void>;
  isAddModalOpen: boolean;
  setIsAddModalOpen: (open: boolean) => void;
}

export const LedgerView: React.FC<LedgerViewProps> = ({
  card,
  transactions,
  categories,
  onAddTransaction,
  onUpdateTransaction,
  onDeleteTransaction,
  isAddModalOpen,
  setIsAddModalOpen,
}) => {
  // Filters
  const [searchQuery, setSearchQuery] = useState("");
  const [typeFilter, setTypeFilter] = useState<"all" | "in" | "out">("all");
  const [selectedCategory, setSelectedCategory] = useState<string>("all");
  const [startDate, setStartDate] = useState("");
  const [endDate, setEndDate] = useState("");
  const [showFilters, setShowFilters] = useState(false);

  // Edit modal state
  const [editingTx, setEditingTx] = useState<Transaction | null>(null);

  // Form states for Add / Edit
  const [formDescription, setFormDescription] = useState("");
  const [formAmountRaw, setFormAmountRaw] = useState("");
  const [formIsIncome, setFormIsIncome] = useState(false);
  const [formCategory, setFormCategory] = useState("Food");
  const [formDate, setFormDate] = useState(todayIso());
  const [formError, setFormError] = useState<string | null>(null);

  // Open Edit Modal
  const openEditModal = (tx: Transaction) => {
    setEditingTx(tx);
    setFormDescription(tx.description);
    setFormAmountRaw(tx.amount.toString());
    setFormIsIncome(tx.is_income);
    setFormCategory(tx.category);
    setFormDate(tx.date);
    setFormError(null);
  };

  // Reset Add Form
  const openAddModal = () => {
    setFormDescription("");
    setFormAmountRaw("");
    setFormIsIncome(false);
    setFormCategory(categories[0]?.name || "Food");
    setFormDate(todayIso());
    setFormError(null);
    setIsAddModalOpen(true);
  };

  // Save new transaction
  const handleSaveAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    const parsedAmount = parsePkrInput(formAmountRaw);
    if (parsedAmount === null || parsedAmount <= 0) {
      setFormError("Please enter a valid amount (e.g. 1,500 or 1500).");
      return;
    }
    if (!formDescription.trim()) {
      setFormError("Please enter a description.");
      return;
    }

    const newTx: Transaction = {
      id: generateUuid(),
      card_id: card.id,
      date: formDate,
      description: formDescription.trim(),
      category: formCategory,
      amount: parsedAmount,
      is_income: formIsIncome,
      auto_generated: false,
      updated_at: new Date().toISOString(),
      deleted_at: null,
    };

    await onAddTransaction(newTx);
    setIsAddModalOpen(false);
  };

  // Save edited transaction
  const handleSaveEdit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingTx) return;

    const parsedAmount = parsePkrInput(formAmountRaw);
    if (parsedAmount === null || parsedAmount <= 0) {
      setFormError("Please enter a valid amount.");
      return;
    }
    if (!formDescription.trim()) {
      setFormError("Please enter a description.");
      return;
    }

    const updated: Transaction = {
      ...editingTx,
      date: formDate,
      description: formDescription.trim(),
      category: formCategory,
      amount: parsedAmount,
      is_income: formIsIncome,
      updated_at: new Date().toISOString(),
    };

    await onUpdateTransaction(updated);
    setEditingTx(null);
  };

  // Delete transaction
  const handleDelete = async () => {
    if (!editingTx) return;
    if (confirm("Are you sure you want to delete this transaction?")) {
      await onDeleteTransaction(editingTx.id);
      setEditingTx(null);
    }
  };

  // Calculate Running Balance Chronologically
  // 1. Sort all non-deleted transactions for this card by date ascending
  // 2. Attach running balance to each
  const { ledgerItems, totalBalance, totalIncome, totalExpense } = useMemo(() => {
    const cardTxs = transactions
      .filter((t) => t.card_id === card.id && !t.deleted_at)
      .sort((a, b) => a.date.localeCompare(b.date));

    let running = card.opening_balance;
    let incomeSum = 0;
    let expenseSum = 0;

    const itemsWithBalance = cardTxs.map((t) => {
      if (t.is_income) {
        running += t.amount;
        incomeSum += t.amount;
      } else {
        running -= t.amount;
        expenseSum += t.amount;
      }
      return {
        ...t,
        runningBalance: running,
      };
    });

    const currentTotal = running;

    // Apply display filters (search, category, type, date range)
    let filtered = itemsWithBalance;

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (t) =>
          t.description.toLowerCase().includes(q) ||
          t.category.toLowerCase().includes(q)
      );
    }

    if (typeFilter === "in") {
      filtered = filtered.filter((t) => t.is_income);
    } else if (typeFilter === "out") {
      filtered = filtered.filter((t) => !t.is_income);
    }

    if (selectedCategory !== "all") {
      filtered = filtered.filter((t) => t.category === selectedCategory);
    }

    if (startDate) {
      filtered = filtered.filter((t) => t.date >= startDate);
    }

    if (endDate) {
      filtered = filtered.filter((t) => t.date <= endDate);
    }

    // Display newest transactions first
    const displayList = [...filtered].reverse();

    return {
      ledgerItems: displayList,
      totalBalance: currentTotal,
      totalIncome: incomeSum,
      totalExpense: expenseSum,
    };
  }, [card, transactions, searchQuery, typeFilter, selectedCategory, startDate, endDate]);

  return (
    <div style={{ paddingBottom: "100px" }}>
      {/* 1. Account Summary Card */}
      <div
        className="card"
        style={{
          margin: "16px",
          background: "linear-gradient(145deg, var(--panel) 0%, var(--panel2) 100%)",
          border: "1px solid var(--border)",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "8px" }}>
          <div>
            <div style={{ fontSize: "12px", color: "var(--muted)", textTransform: "uppercase", letterSpacing: "0.05em" }}>
              {card.name} {card.is_primary && "★ Primary"}
            </div>
            <div className="mono" style={{ fontSize: "28px", fontWeight: 700, color: "var(--text)", marginTop: "4px" }}>
              {formatPkr(totalBalance)}
            </div>
          </div>
          <button
            onClick={openAddModal}
            style={{
              background: "var(--in-btn-bg)",
              color: "var(--in)",
              border: "1px solid rgba(63, 205, 168, 0.3)",
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

        <div style={{ display: "flex", gap: "16px", marginTop: "14px", paddingTop: "12px", borderTop: "1px solid var(--border)" }}>
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: "11px", color: "var(--muted)", display: "flex", alignItems: "center", gap: "4px" }}>
              <ArrowDownLeft size={12} color="var(--in)" /> Money In
            </div>
            <div className="mono" style={{ fontSize: "14px", color: "var(--in)", fontWeight: 600, marginTop: "2px" }}>
              {formatPkrWhole(totalIncome)}
            </div>
          </div>
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: "11px", color: "var(--muted)", display: "flex", alignItems: "center", gap: "4px" }}>
              <ArrowUpRight size={12} color="var(--out)" /> Money Out
            </div>
            <div className="mono" style={{ fontSize: "14px", color: "var(--out)", fontWeight: 600, marginTop: "2px" }}>
              {formatPkrWhole(totalExpense)}
            </div>
          </div>
        </div>
      </div>

      {/* 2. Search & Filter Bar */}
      <div style={{ padding: "0 16px 12px 16px", display: "flex", gap: "8px" }}>
        <div style={{ position: "relative", flex: 1 }}>
          <Search size={16} style={{ position: "absolute", left: "12px", top: "50%", transform: "translateY(-50%)", color: "var(--muted)" }} />
          <input
            type="text"
            placeholder="Search transactions..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            style={{
              width: "100%",
              padding: "8px 12px 8px 36px",
              fontSize: "14px",
              borderRadius: "var(--radius-btn)",
            }}
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              style={{
                position: "absolute",
                right: "8px",
                top: "50%",
                transform: "translateY(-50%)",
                background: "transparent",
                color: "var(--muted)",
                padding: "4px",
              }}
            >
              <X size={14} />
            </button>
          )}
        </div>

        <button
          onClick={() => setShowFilters(!showFilters)}
          style={{
            background: showFilters ? "var(--accent-dim)" : "var(--panel2)",
            color: showFilters ? "var(--accent)" : "var(--muted)",
            border: `1px solid ${showFilters ? "var(--accent)" : "var(--border)"}`,
            padding: "8px 12px",
            borderRadius: "var(--radius-btn)",
          }}
          aria-label="Toggle filters"
        >
          <Filter size={16} />
        </button>
      </div>

      {/* Expanded Filter Panel */}
      {showFilters && (
        <div
          style={{
            margin: "0 16px 16px 16px",
            padding: "14px",
            background: "var(--panel2)",
            border: "1px solid var(--border)",
            borderRadius: "var(--radius-card)",
            display: "flex",
            flexDirection: "column",
            gap: "12px",
          }}
        >
          {/* In / Out Pills */}
          <div>
            <div style={{ fontSize: "11px", color: "var(--muted)", marginBottom: "6px", textTransform: "uppercase" }}>
              Flow
            </div>
            <div style={{ display: "flex", gap: "6px" }}>
              {(["all", "in", "out"] as const).map((t) => (
                <button
                  key={t}
                  onClick={() => setTypeFilter(t)}
                  style={{
                    flex: 1,
                    padding: "6px",
                    fontSize: "12px",
                    fontWeight: 600,
                    borderRadius: "var(--radius-btn)",
                    background: typeFilter === t ? "var(--accent)" : "var(--panel)",
                    color: typeFilter === t ? "#fff" : "var(--muted)",
                    border: `1px solid ${typeFilter === t ? "var(--accent)" : "var(--border)"}`,
                  }}
                >
                  {t === "all" ? "All" : t === "in" ? "Income" : "Expense"}
                </button>
              ))}
            </div>
          </div>

          {/* Category Dropdown */}
          <div>
            <div style={{ fontSize: "11px", color: "var(--muted)", marginBottom: "6px", textTransform: "uppercase" }}>
              Category
            </div>
            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value)}
              style={{ width: "100%", padding: "8px 12px", fontSize: "13px" }}
            >
              <option value="all">All Categories</option>
              {categories.map((c) => (
                <option key={c.id} value={c.name}>
                  {c.name}
                </option>
              ))}
            </select>
          </div>

          {/* Date Range Filters (Allows Backdating / any range) */}
          <div style={{ display: "flex", gap: "8px" }}>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: "11px", color: "var(--muted)", marginBottom: "4px" }}>From</div>
              <input
                type="date"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                style={{ width: "100%", padding: "6px 8px", fontSize: "12px" }}
              />
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: "11px", color: "var(--muted)", marginBottom: "4px" }}>To</div>
              <input
                type="date"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
                style={{ width: "100%", padding: "6px 8px", fontSize: "12px" }}
              />
            </div>
          </div>

          {(typeFilter !== "all" || selectedCategory !== "all" || startDate || endDate) && (
            <button
              onClick={() => {
                setTypeFilter("all");
                setSelectedCategory("all");
                setStartDate("");
                setEndDate("");
              }}
              style={{
                alignSelf: "flex-end",
                background: "transparent",
                color: "var(--out)",
                fontSize: "12px",
                padding: "4px 8px",
              }}
            >
              Reset Filters
            </button>
          )}
        </div>
      )}

      {/* 3. Stacked Mobile Transactions List (Tap-to-Edit) */}
      <div style={{ padding: "0 16px" }}>
        <div style={{ fontSize: "13px", fontWeight: 600, color: "var(--muted)", marginBottom: "8px", textTransform: "uppercase", letterSpacing: "0.04em" }}>
          Transactions ({ledgerItems.length})
        </div>

        {ledgerItems.length === 0 ? (
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
            No transactions found. Tap <strong>+ New</strong> to record one.
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            {ledgerItems.map((tx) => (
              <div
                key={tx.id}
                onClick={() => openEditModal(tx)}
                style={{
                  background: "var(--panel)",
                  border: "1px solid var(--border)",
                  borderRadius: "var(--radius-card)",
                  padding: "12px 14px",
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  cursor: "pointer",
                  transition: "background-color 0.15s",
                }}
              >
                {/* Left: Description, Category, Date */}
                <div style={{ display: "flex", flexDirection: "column", gap: "4px", maxWidth: "65%" }}>
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <span
                      style={{
                        fontWeight: 600,
                        fontSize: "14px",
                        color: "var(--text)",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {tx.description}
                    </span>
                    <span
                      className="badge badge-neutral"
                      style={{ fontSize: "10px", padding: "1px 6px" }}
                    >
                      {tx.category}
                    </span>
                  </div>

                  <div style={{ fontSize: "12px", color: "var(--muted)" }}>
                    {formatDate(tx.date)}
                    {tx.auto_generated && (
                      <span style={{ marginLeft: "6px", color: "var(--accent)" }}>• Auto</span>
                    )}
                  </div>
                </div>

                {/* Right: Color-Coded Amount & Running Balance */}
                <div style={{ textAlign: "right" }}>
                  <div
                    className="mono"
                    style={{
                      fontSize: "15px",
                      fontWeight: 700,
                      color: tx.is_income ? "var(--in)" : "var(--out)",
                    }}
                  >
                    {tx.is_income ? "+" : "-"}
                    {formatPkr(tx.amount)}
                  </div>

                  <div
                    className="mono"
                    style={{ fontSize: "11px", color: "var(--muted)", marginTop: "2px" }}
                  >
                    Bal: {formatPkr(tx.runningBalance)}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Anchor: Opening Balance Card */}
        <div
          style={{
            marginTop: "16px",
            padding: "12px 14px",
            background: "var(--panel2)",
            border: "1px dashed var(--border)",
            borderRadius: "var(--radius-card)",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            opacity: 0.8,
          }}
        >
          <div>
            <div style={{ fontSize: "13px", fontWeight: 600, color: "var(--muted)" }}>
              {card.opening_balance_description || "Opening Balance"}
            </div>
            <div style={{ fontSize: "11px", color: "var(--faint)" }}>
              {card.opening_balance_date ? formatDate(card.opening_balance_date) : "Account Inception"}
            </div>
          </div>
          <div className="mono" style={{ fontSize: "14px", fontWeight: 600, color: "var(--muted)" }}>
            {formatPkr(card.opening_balance)}
          </div>
        </div>
      </div>

      {/* 4. Add Transaction Modal */}
      {isAddModalOpen && (
        <div className="modal-overlay" onClick={() => setIsAddModalOpen(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-handle" />
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
              <h3 style={{ margin: 0, fontSize: "18px" }}>Add Transaction</h3>
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
              {/* Type Switcher: Income / Expense */}
              <div className="form-group">
                <label className="form-label">Type</label>
                <div style={{ display: "flex", gap: "8px" }}>
                  <button
                    type="button"
                    onClick={() => setFormIsIncome(false)}
                    style={{
                      flex: 1,
                      padding: "10px",
                      borderRadius: "var(--radius-btn)",
                      background: !formIsIncome ? "var(--out)" : "var(--panel2)",
                      color: !formIsIncome ? "#ffffff" : "var(--muted)",
                      fontWeight: 600,
                    }}
                  >
                    Money Out
                  </button>
                  <button
                    type="button"
                    onClick={() => setFormIsIncome(true)}
                    style={{
                      flex: 1,
                      padding: "10px",
                      borderRadius: "var(--radius-btn)",
                      background: formIsIncome ? "var(--in)" : "var(--panel2)",
                      color: formIsIncome ? "#11141A" : "var(--muted)",
                      fontWeight: 600,
                    }}
                  >
                    Money In
                  </button>
                </div>
              </div>

              {/* PKR Amount (Comma-tolerant) */}
              <div className="form-group">
                <label className="form-label">Amount (PKR)</label>
                <input
                  type="text"
                  inputMode="decimal"
                  placeholder="e.g. 5,000 or 5000.50"
                  value={formAmountRaw}
                  onChange={(e) => setFormAmountRaw(e.target.value)}
                  className="form-input mono"
                  autoFocus
                  required
                />
              </div>

              {/* Description */}
              <div className="form-group">
                <label className="form-label">Description</label>
                <input
                  type="text"
                  placeholder="e.g. Grocery Store"
                  value={formDescription}
                  onChange={(e) => setFormDescription(e.target.value)}
                  className="form-input"
                  required
                />
              </div>

              {/* Category */}
              <div className="form-group">
                <label className="form-label">Category</label>
                <select
                  value={formCategory}
                  onChange={(e) => setFormCategory(e.target.value)}
                  className="form-input"
                >
                  {categories.map((c) => (
                    <option key={c.id} value={c.name}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </div>

              {/* Date (Supports Backdating without restrictions) */}
              <div className="form-group">
                <label className="form-label">Date (Backdating Allowed)</label>
                <input
                  type="date"
                  value={formDate}
                  onChange={(e) => setFormDate(e.target.value)}
                  className="form-input"
                  required
                />
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
                Record Transaction
              </button>
            </form>
          </div>
        </div>
      )}

      {/* 5. Tap-to-Edit Transaction Modal */}
      {editingTx && (
        <div className="modal-overlay" onClick={() => setEditingTx(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-handle" />
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
              <h3 style={{ margin: 0, fontSize: "18px" }}>Edit Transaction</h3>
              <button
                onClick={() => setEditingTx(null)}
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
              {/* Type Switcher */}
              <div className="form-group">
                <label className="form-label">Type</label>
                <div style={{ display: "flex", gap: "8px" }}>
                  <button
                    type="button"
                    onClick={() => setFormIsIncome(false)}
                    style={{
                      flex: 1,
                      padding: "10px",
                      borderRadius: "var(--radius-btn)",
                      background: !formIsIncome ? "var(--out)" : "var(--panel2)",
                      color: !formIsIncome ? "#ffffff" : "var(--muted)",
                      fontWeight: 600,
                    }}
                  >
                    Money Out
                  </button>
                  <button
                    type="button"
                    onClick={() => setFormIsIncome(true)}
                    style={{
                      flex: 1,
                      padding: "10px",
                      borderRadius: "var(--radius-btn)",
                      background: formIsIncome ? "var(--in)" : "var(--panel2)",
                      color: formIsIncome ? "#11141A" : "var(--muted)",
                      fontWeight: 600,
                    }}
                  >
                    Money In
                  </button>
                </div>
              </div>

              {/* PKR Amount (Comma-tolerant) */}
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

              {/* Description */}
              <div className="form-group">
                <label className="form-label">Description</label>
                <input
                  type="text"
                  value={formDescription}
                  onChange={(e) => setFormDescription(e.target.value)}
                  className="form-input"
                  required
                />
              </div>

              {/* Category */}
              <div className="form-group">
                <label className="form-label">Category</label>
                <select
                  value={formCategory}
                  onChange={(e) => setFormCategory(e.target.value)}
                  className="form-input"
                >
                  {categories.map((c) => (
                    <option key={c.id} value={c.name}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </div>

              {/* Date */}
              <div className="form-group">
                <label className="form-label">Date (Backdating Allowed)</label>
                <input
                  type="date"
                  value={formDate}
                  onChange={(e) => setFormDate(e.target.value)}
                  className="form-input"
                  required
                />
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

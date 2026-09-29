import React from "react";
import { CreditCard, Calendar, Target } from "lucide-react";

export type ActiveTab = "ledger" | "subscriptions" | "savings";

interface BottomNavProps {
  activeTab: ActiveTab;
  onTabChange: (tab: ActiveTab) => void;
}

export const BottomNav: React.FC<BottomNavProps> = ({ activeTab, onTabChange }) => {
  return (
    <nav className="bottom-nav" aria-label="Bottom Navigation">
      <button
        type="button"
        className={`nav-tab ${activeTab === "ledger" ? "active" : ""}`}
        onClick={() => onTabChange("ledger")}
      >
        <CreditCard size={22} strokeWidth={activeTab === "ledger" ? 2.5 : 1.8} />
        <span>Bank Ledgers</span>
      </button>

      <button
        type="button"
        className={`nav-tab ${activeTab === "subscriptions" ? "active" : ""}`}
        onClick={() => onTabChange("subscriptions")}
      >
        <Calendar size={22} strokeWidth={activeTab === "subscriptions" ? 2.5 : 1.8} />
        <span>Subscriptions</span>
      </button>

      <button
        type="button"
        className={`nav-tab ${activeTab === "savings" ? "active" : ""}`}
        onClick={() => onTabChange("savings")}
      >
        <Target size={22} strokeWidth={activeTab === "savings" ? 2.5 : 1.8} />
        <span>Savings Plans</span>
      </button>
    </nav>
  );
};

import React, { useState, useEffect, useCallback } from "react";
import { AuthView } from "./components/AuthView";
import { TopBar } from "./components/TopBar";
import { BottomNav, ActiveTab } from "./components/BottomNav";
import { HomeScreenBanner } from "./components/HomeScreenBanner";
import { NotificationPrompt } from "./components/NotificationPrompt";
import { LedgerView } from "./components/LedgerView";
import { SubscriptionsView } from "./components/SubscriptionsView";
import { SavingsPlansView } from "./components/SavingsPlansView";
import { SettingsModal } from "./components/SettingsModal";
import { Card, Category, SavingsPlan, Subscription, Transaction } from "./types";
import {
  getAllCards,
  getAllCategories,
  getAllSavingsPlans,
  getAllSubscriptions,
  getAllTransactions,
  getMeta,
  putCard,
  putCategory,
  putSavingsPlan,
  putSubscription,
  putTransaction,
  setMeta,
} from "./lib/db";
import { getSupabase } from "./lib/supabase";
import { performSync } from "./lib/sync";
import { generateUuid } from "./lib/utils";

export const App: React.FC = () => {
  // Session & Auth state
  const [session, setSession] = useState<any>(null);
  const [authChecked, setAuthChecked] = useState(false);

  // App Navigation
  const [activeTab, setActiveTab] = useState<ActiveTab>("ledger");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);

  // App Data
  const [cards, setCards] = useState<Card[]>([]);
  const [selectedCardId, setSelectedCardId] = useState<string>("");
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [plans, setPlans] = useState<SavingsPlan[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);

  // Sync & Network status
  const [syncStatus, setSyncStatus] = useState<"synced" | "syncing" | "offline" | "error">(
    navigator.onLine ? "synced" : "offline"
  );

  // 1. Initial Auth Check
  useEffect(() => {
    const supabase = getSupabase();
    if (!supabase) {
      setAuthChecked(true);
      return;
    }

    supabase.auth.getSession().then(({ data: { session } }) => {
      setSession(session);
      setAuthChecked(true);
    });

    const {
      data: { subscription },
    } = supabase.auth.onAuthStateChange((_event, session) => {
      setSession(session);
    });

    return () => subscription.unsubscribe();
  }, []);

  // 2. Load Local IndexedDB Data
  const loadLocalData = useCallback(async () => {
    try {
      const localCards = await getAllCards();
      const localTxs = await getAllTransactions();
      const localSubs = await getAllSubscriptions();
      const localPlans = await getAllSavingsPlans();
      const localCats = await getAllCategories();

      // Seed defaults if brand new local DB
      if (localCards.length === 0) {
        const defaultCardId = generateUuid();
        const defaultCard: Card = {
          id: defaultCardId,
          name: "Main Card",
          is_primary: true,
          opening_balance: 0,
          opening_balance_description: "Opening Balance",
          opening_balance_date: null,
          updated_at: new Date().toISOString(),
          deleted_at: null,
        };
        await putCard(defaultCard, true);
        localCards.push(defaultCard);
      }

      if (localCats.length === 0) {
        const defaultCategoryNames = ["Food", "Income", "Subscription", "Transport", "Shopping", "Other"];
        for (const name of defaultCategoryNames) {
          const cat: Category = {
            id: generateUuid(),
            name,
            is_default: true,
            updated_at: new Date().toISOString(),
            deleted_at: null,
          };
          await putCategory(cat, true);
          localCats.push(cat);
        }
      }

      setCards(localCards);
      setTransactions(localTxs);
      setSubscriptions(localSubs);
      setPlans(localPlans);
      setCategories(localCats);

      // Restore active selected card or pick primary
      const storedCardId = await getMeta("selected_card_id");
      if (storedCardId && localCards.some((c) => c.id === storedCardId)) {
        setSelectedCardId(storedCardId);
      } else {
        const primary = localCards.find((c) => c.is_primary) || localCards[0];
        if (primary) setSelectedCardId(primary.id);
      }
    } catch (err) {
      console.error("Failed to load local IndexedDB data:", err);
    }
  }, []);

  // 3. Trigger Sync Engine
  const triggerSync = useCallback(async () => {
    if (!session?.user?.id || !navigator.onLine) {
      setSyncStatus(navigator.onLine ? "synced" : "offline");
      return;
    }

    setSyncStatus("syncing");
    const result = await performSync(session.user.id);

    if (result.success) {
      setSyncStatus("synced");
      // Reload UI state from IndexedDB after successful pull
      await loadLocalData();
    } else {
      setSyncStatus("error");
      console.warn("Sync error:", result.error);
    }
  }, [session, loadLocalData]);

  // Load data when session is established
  useEffect(() => {
    if (session) {
      loadLocalData().then(() => {
        triggerSync();
      });
    }
  }, [session, loadLocalData, triggerSync]);

  // 4. Online/Offline & Auto-Sync Interval
  useEffect(() => {
    const handleOnline = () => {
      setSyncStatus("synced");
      triggerSync();
    };
    const handleOffline = () => {
      setSyncStatus("offline");
    };

    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);

    // Auto-sync every 60s while tab/app is open
    const interval = setInterval(() => {
      if (document.visibilityState === "visible") {
        triggerSync();
      }
    }, 60000);

    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
      clearInterval(interval);
    };
  }, [triggerSync]);

  // Card Selection Handler
  const handleSelectCard = async (cardId: string) => {
    setSelectedCardId(cardId);
    await setMeta("selected_card_id", cardId);
  };

  // CRUD Handlers for Transactions
  const handleAddTransaction = async (tx: Transaction) => {
    await putTransaction(tx, true);
    await loadLocalData();
    triggerSync();
  };

  const handleUpdateTransaction = async (tx: Transaction) => {
    await putTransaction(tx, true);
    await loadLocalData();
    triggerSync();
  };

  const handleDeleteTransaction = async (txId: string) => {
    const existing = transactions.find((t) => t.id === txId);
    if (existing) {
      const softDeleted: Transaction = {
        ...existing,
        deleted_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      await putTransaction(softDeleted, true);
      await loadLocalData();
      triggerSync();
    }
  };

  // CRUD Handlers for Subscriptions
  const handleAddSubscription = async (sub: Subscription) => {
    await putSubscription(sub, true);
    await loadLocalData();
    triggerSync();
  };

  const handleUpdateSubscription = async (sub: Subscription) => {
    await putSubscription(sub, true);
    await loadLocalData();
    triggerSync();
  };

  const handleDeleteSubscription = async (subId: string) => {
    const existing = subscriptions.find((s) => s.id === subId);
    if (existing) {
      const softDeleted: Subscription = {
        ...existing,
        deleted_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      await putSubscription(softDeleted, true);
      await loadLocalData();
      triggerSync();
    }
  };

  // CRUD Handlers for Savings Plans
  const handleAddPlan = async (plan: SavingsPlan) => {
    await putSavingsPlan(plan, true);
    await loadLocalData();
    triggerSync();
  };

  const handleUpdatePlan = async (plan: SavingsPlan) => {
    await putSavingsPlan(plan, true);
    await loadLocalData();
    triggerSync();
  };

  const handleDeletePlan = async (planId: string) => {
    const existing = plans.find((p) => p.id === planId);
    if (existing) {
      const softDeleted: SavingsPlan = {
        ...existing,
        deleted_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      await putSavingsPlan(softDeleted, true);
      await loadLocalData();
      triggerSync();
    }
  };

  // CRUD Handlers for Cards & Categories
  const handleAddCard = async (newCard: Card) => {
    await putCard(newCard, true);
    await loadLocalData();
    setSelectedCardId(newCard.id);
    triggerSync();
  };

  const handleAddCategory = async (newCat: Category) => {
    await putCategory(newCat, true);
    await loadLocalData();
    triggerSync();
  };

  const handleLogOut = async () => {
    const supabase = getSupabase();
    if (supabase) {
      await supabase.auth.signOut();
    }
    setSession(null);
    setIsSettingsOpen(false);
  };

  if (!authChecked) {
    return (
      <div style={{ minHeight: "100vh", display: "flex", alignItems: "center", justifyContent: "center" }}>
        <div style={{ color: "var(--muted)" }}>Loading Savings Tracker...</div>
      </div>
    );
  }

  if (!session) {
    return <AuthView onAuthSuccess={() => triggerSync()} />;
  }

  const activeCard = cards.find((c) => c.id === selectedCardId) || cards[0];

  const getTabTitle = () => {
    switch (activeTab) {
      case "ledger":
        return activeCard ? activeCard.name : "Bank Ledgers";
      case "subscriptions":
        return "Subscriptions";
      case "savings":
        return "Savings Goals";
    }
  };

  return (
    <div style={{ minHeight: "100vh", display: "flex", flexDirection: "column" }}>
      {/* Top Bar with Card Switcher & Sync Status */}
      <TopBar
        cards={cards}
        selectedCardId={selectedCardId}
        onSelectCard={handleSelectCard}
        syncStatus={syncStatus}
        onTriggerSync={triggerSync}
        onOpenSettings={() => setIsSettingsOpen(true)}
        onQuickAdd={() => setIsAddModalOpen(true)}
        tabTitle={getTabTitle()}
      />

      <main style={{ flex: 1, maxWidth: "var(--content-max-width)", width: "100%", margin: "0 auto" }}>
        {/* iOS Add to Home Screen first-visit banner */}
        <HomeScreenBanner />

        {/* Web Push Alerts Prompt for iPhone */}
        <NotificationPrompt userId={session.user.id} />

        {/* Tab Views */}
        {activeTab === "ledger" && activeCard && (
          <LedgerView
            card={activeCard}
            transactions={transactions}
            categories={categories}
            onAddTransaction={handleAddTransaction}
            onUpdateTransaction={handleUpdateTransaction}
            onDeleteTransaction={handleDeleteTransaction}
            isAddModalOpen={isAddModalOpen}
            setIsAddModalOpen={setIsAddModalOpen}
          />
        )}

        {activeTab === "subscriptions" && (
          <SubscriptionsView
            subscriptions={subscriptions}
            cards={cards}
            onAddSubscription={handleAddSubscription}
            onUpdateSubscription={handleUpdateSubscription}
            onDeleteSubscription={handleDeleteSubscription}
            isAddModalOpen={isAddModalOpen}
            setIsAddModalOpen={setIsAddModalOpen}
          />
        )}

        {activeTab === "savings" && (
          <SavingsPlansView
            plans={plans}
            cards={cards}
            transactions={transactions}
            subscriptions={subscriptions}
            onAddPlan={handleAddPlan}
            onUpdatePlan={handleUpdatePlan}
            onDeletePlan={handleDeletePlan}
            isAddModalOpen={isAddModalOpen}
            setIsAddModalOpen={setIsAddModalOpen}
          />
        )}
      </main>

      {/* Mobile Bottom Tab Bar */}
      <BottomNav activeTab={activeTab} onTabChange={setActiveTab} />

      {/* Settings Modal */}
      {isSettingsOpen && (
        <SettingsModal
          userEmail={session.user.email || ""}
          userId={session.user.id}
          cards={cards}
          categories={categories}
          onAddCard={handleAddCard}
          onAddCategory={handleAddCategory}
          onTriggerSync={triggerSync}
          onLogOut={handleLogOut}
          onClose={() => setIsSettingsOpen(false)}
        />
      )}
    </div>
  );
};

export default App;

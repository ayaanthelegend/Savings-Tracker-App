import { openDB, DBSchema, IDBPDatabase } from "idb";
import { Card, Category, SavingsPlan, Subscription, SyncQueueItem, TableName, Transaction } from "../types";

interface SavingsTrackerDB extends DBSchema {
  cards: {
    key: string;
    value: Card;
    indexes: { "by-updated": string };
  };
  transactions: {
    key: string;
    value: Transaction;
    indexes: { "by-card": string; "by-date": string; "by-updated": string };
  };
  subscriptions: {
    key: string;
    value: Subscription;
    indexes: { "by-card": string; "by-updated": string };
  };
  savings_plans: {
    key: string;
    value: SavingsPlan;
    indexes: { "by-updated": string };
  };
  categories: {
    key: string;
    value: Category;
    indexes: { "by-updated": string };
  };
  sync_watermarks: {
    key: string;
    value: { table: TableName; watermark: string };
  };
  sync_queue: {
    key: string;
    value: SyncQueueItem;
    indexes: { "by-timestamp": number };
  };
  meta: {
    key: string;
    value: { key: string; value: any };
  };
}

let dbPromise: Promise<IDBPDatabase<SavingsTrackerDB>> | null = null;

export function getDB(): Promise<IDBPDatabase<SavingsTrackerDB>> {
  if (!dbPromise) {
    dbPromise = openDB<SavingsTrackerDB>("savings_tracker_db", 1, {
      upgrade(db) {
        // Cards store
        const cardStore = db.createObjectStore("cards", { keyPath: "id" });
        cardStore.createIndex("by-updated", "updated_at");

        // Transactions store
        const txStore = db.createObjectStore("transactions", { keyPath: "id" });
        txStore.createIndex("by-card", "card_id");
        txStore.createIndex("by-date", "date");
        txStore.createIndex("by-updated", "updated_at");

        // Subscriptions store
        const subStore = db.createObjectStore("subscriptions", { keyPath: "id" });
        subStore.createIndex("by-card", "card_id");
        subStore.createIndex("by-updated", "updated_at");

        // Savings Plans store
        const planStore = db.createObjectStore("savings_plans", { keyPath: "id" });
        planStore.createIndex("by-updated", "updated_at");

        // Categories store
        const catStore = db.createObjectStore("categories", { keyPath: "id" });
        catStore.createIndex("by-updated", "updated_at");

        // Sync watermarks
        db.createObjectStore("sync_watermarks", { keyPath: "table" });

        // Sync queue for offline mutations
        const queueStore = db.createObjectStore("sync_queue", { keyPath: "id" });
        queueStore.createIndex("by-timestamp", "timestamp");

        // Key-value metadata store
        db.createObjectStore("meta", { keyPath: "key" });
      },
    });
  }
  return dbPromise;
}

// --- CRUD Helpers for local IndexedDB cache ---

export async function getAllCards(): Promise<Card[]> {
  const db = await getDB();
  const all = await db.getAll("cards");
  return all.filter((c) => !c.deleted_at);
}

export async function putCard(card: Card, enqueueSync = true): Promise<void> {
  const db = await getDB();
  await db.put("cards", card);
  if (enqueueSync) {
    await enqueueMutation("cards", card);
  }
}

export async function getAllTransactions(cardId?: string): Promise<Transaction[]> {
  const db = await getDB();
  let all: Transaction[];
  if (cardId) {
    all = await db.getAllFromIndex("transactions", "by-card", cardId);
  } else {
    all = await db.getAll("transactions");
  }
  return all.filter((t) => !t.deleted_at);
}

export async function putTransaction(tx: Transaction, enqueueSync = true): Promise<void> {
  const db = await getDB();
  await db.put("transactions", tx);
  if (enqueueSync) {
    await enqueueMutation("transactions", tx);
  }
}

export async function getAllSubscriptions(): Promise<Subscription[]> {
  const db = await getDB();
  const all = await db.getAll("subscriptions");
  return all.filter((s) => !s.deleted_at);
}

export async function putSubscription(sub: Subscription, enqueueSync = true): Promise<void> {
  const db = await getDB();
  await db.put("subscriptions", sub);
  if (enqueueSync) {
    await enqueueMutation("subscriptions", sub);
  }
}

export async function getAllSavingsPlans(): Promise<SavingsPlan[]> {
  const db = await getDB();
  const all = await db.getAll("savings_plans");
  return all.filter((p) => !p.deleted_at);
}

export async function putSavingsPlan(plan: SavingsPlan, enqueueSync = true): Promise<void> {
  const db = await getDB();
  await db.put("savings_plans", plan);
  if (enqueueSync) {
    await enqueueMutation("savings_plans", plan);
  }
}

export async function getAllCategories(): Promise<Category[]> {
  const db = await getDB();
  const all = await db.getAll("categories");
  return all.filter((c) => !c.deleted_at);
}

export async function putCategory(category: Category, enqueueSync = true): Promise<void> {
  const db = await getDB();
  await db.put("categories", category);
  if (enqueueSync) {
    await enqueueMutation("categories", category);
  }
}

// --- Sync Queue Helpers ---

export async function enqueueMutation(table: TableName, record: any): Promise<void> {
  const db = await getDB();
  const item: SyncQueueItem = {
    id: `${table}:${record.id}`,
    table,
    record,
    timestamp: Date.now(),
  };
  await db.put("sync_queue", item);
}

export async function getSyncQueue(): Promise<SyncQueueItem[]> {
  const db = await getDB();
  return db.getAllFromIndex("sync_queue", "by-timestamp");
}

export async function removeSyncQueueItem(id: string): Promise<void> {
  const db = await getDB();
  await db.delete("sync_queue", id);
}

// --- Watermark Helpers ---

export async function getWatermark(table: TableName): Promise<string | undefined> {
  const db = await getDB();
  const entry = await db.get("sync_watermarks", table);
  return entry?.watermark;
}

export async function setWatermark(table: TableName, watermark: string): Promise<void> {
  const db = await getDB();
  await db.put("sync_watermarks", { table, watermark });
}

// --- Metadata Helpers ---

export async function getMeta(key: string): Promise<any> {
  const db = await getDB();
  const entry = await db.get("meta", key);
  return entry?.value;
}

export async function setMeta(key: string, value: any): Promise<void> {
  const db = await getDB();
  await db.put("meta", { key, value });
}

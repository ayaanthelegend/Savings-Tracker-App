import { TableName } from "../types";
import {
  getDB,
  getSyncQueue,
  removeSyncQueueItem,
  getWatermark,
  setWatermark,
  putCard,
  putTransaction,
  putSubscription,
  putSavingsPlan,
  putCategory,
  getAllSubscriptions,
  getAllSavingsPlans,
  getAllTransactions,
} from "./db";
import { getSupabase } from "./supabase";
import { advanceSubscriptionDate, computePlanProgress, generateUuid, todayIso } from "./utils";

const TABLES: TableName[] = [
  "cards",
  "transactions",
  "subscriptions",
  "savings_plans",
  "categories",
];

export interface SyncResult {
  success: boolean;
  pushed: number;
  pulled: number;
  error?: string;
}

let isSyncing = false;

/**
 * Pushes pending mutations from IndexedDB queue to Supabase,
 * then pulls new/updated rows from Supabase into IndexedDB.
 */
export async function performSync(userId: string): Promise<SyncResult> {
  if (isSyncing) {
    return { success: true, pushed: 0, pulled: 0 };
  }

  const supabase = getSupabase();
  if (!supabase) {
    return { success: false, pushed: 0, pulled: 0, error: "Supabase client not configured." };
  }

  isSyncing = true;
  let pushedCount = 0;
  let pulledCount = 0;

  try {
    const db = await getDB();

    // 1. Drain pending sync queue (Push)
    const queue = await getSyncQueue();
    for (const item of queue) {
      try {
        const payload = {
          ...item.record,
          user_id: userId,
        };

        const { error } = await supabase
          .from(item.table)
          .upsert(payload, { onConflict: "id" });

        if (error) {
          console.error(`Failed to push to ${item.table}:`, error);
          // If error is network, break and retry later
          break;
        }

        await removeSyncQueueItem(item.id);
        pushedCount++;
      } catch (err) {
        console.error("Sync push error:", err);
        break;
      }
    }

    // 2. Pull latest rows for each table using per-table watermarks
    for (const table of TABLES) {
      const watermark = await getWatermark(table);
      let query = supabase.from(table).select("*").order("updated_at", { ascending: true });

      if (watermark) {
        query = query.gt("updated_at", watermark);
      }

      const { data, error } = await query;
      if (error) {
        console.error(`Error pulling from ${table}:`, error);
        continue;
      }

      if (data && data.length > 0) {
        let maxUpdated = watermark || "";

        for (const remote of data) {
          if (remote.updated_at > maxUpdated) {
            maxUpdated = remote.updated_at;
          }

          // Conflict resolution: Check if local copy exists
          const local = await db.get(table as any, remote.id);
          if (!local) {
            // New record from server
            await applyRemoteRecord(table, remote);
            pulledCount++;
          } else {
            // Compare timestamps: Last-Write-Wins (Server wins ties)
            const localUpdated = (local as any).updated_at || "";
            const remoteUpdated = remote.updated_at || "";

            if (remoteUpdated >= localUpdated) {
              await applyRemoteRecord(table, remote);
              pulledCount++;
            }
          }
        }

        if (maxUpdated) {
          await setWatermark(table, maxUpdated);
        }

        // Clean up any local placeholder cards that have no transactions and do not exist on server
        if (table === "cards") {
          const allLocalCards = await db.getAll("cards");
          const serverCards = await supabase.from("cards").select("id");
          if (serverCards.data && serverCards.data.length > 0) {
            const serverCardIds = new Set(serverCards.data.map((c: any) => c.id));
            for (const lc of allLocalCards) {
              if (!serverCardIds.has(lc.id) && lc.opening_balance === 0) {
                const txs = await db.getAllFromIndex("transactions", "by-card", lc.id);
                if (txs.length === 0) {
                  await db.delete("cards", lc.id);
                }
              }
            }
          }
        }
      }
    }

    // 3. Catch up due subscriptions
    await processDueSubscriptionsLocally();

    // 4. Check savings plan expirations
    await checkSavingsExpirationsLocally();

    return { success: true, pushed: pushedCount, pulled: pulledCount };
  } catch (err) {
    console.error("PerformSync failed:", err);
    return { success: false, pushed: pushedCount, pulled: pulledCount, error: String(err) };
  } finally {
    isSyncing = false;
  }
}

async function applyRemoteRecord(table: TableName, record: any) {
  switch (table) {
    case "cards":
      await putCard(record, false);
      break;
    case "transactions":
      await putTransaction(record, false);
      break;
    case "subscriptions":
      await putSubscription(record, false);
      break;
    case "savings_plans":
      await putSavingsPlan(record, false);
      break;
    case "categories":
      await putCategory(record, false);
      break;
  }
}

/**
 * Checks all active subscriptions against today.
 * Generates Money Out transactions for past-due dates and advances next_due_date.
 */
export async function processDueSubscriptionsLocally(): Promise<void> {
  const subscriptions = await getAllSubscriptions();
  const today = todayIso();

  for (const sub of subscriptions) {
    if (sub.is_paused || sub.deleted_at) continue;

    let nextDue = sub.next_due_date;
    let cyclesMissed = 0;
    let updated = false;

    while (nextDue <= today && cyclesMissed < 100) {
      // Create auto-generated transaction
      const nowIso = new Date().toISOString();
      const tx = {
        id: generateUuid(),
        card_id: sub.card_id,
        date: nextDue,
        description: `Subscription: ${sub.name}`,
        category: "Subscription",
        amount: sub.amount,
        is_income: false,
        auto_generated: true,
        updated_at: nowIso,
        deleted_at: null,
      };

      await putTransaction(tx, true);
      nextDue = advanceSubscriptionDate(nextDue, sub.billing_cycle);
      updated = true;
      cyclesMissed++;
    }

    if (updated) {
      const updatedSub = {
        ...sub,
        next_due_date: nextDue,
        updated_at: new Date().toISOString(),
      };
      await putSubscription(updatedSub, true);
    }
  }
}

/**
 * Checks active savings plans against today.
 * If deadline has passed, computes final results and marks inactive.
 */
export async function checkSavingsExpirationsLocally(): Promise<void> {
  const plans = await getAllSavingsPlans();
  const today = todayIso();
  const allTxs = await getAllTransactions();

  for (const plan of plans) {
    if (plan.is_active && !plan.deleted_at && plan.deadline <= today) {
      const calc = computePlanProgress(plan, allTxs, 0, today, plan.deduct_overspending);
      const updatedPlan = {
        ...plan,
        is_active: false,
        updated_at: new Date().toISOString(),
      };
      await putSavingsPlan(updatedPlan, true);
    }
  }
}

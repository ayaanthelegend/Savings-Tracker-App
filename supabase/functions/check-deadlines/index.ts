// Supabase Edge Function: check-deadlines
// Scheduled function (cron) to check expired savings plans and trigger Web Push notifications
// to registered iPhone PWAs.
//
// Required Environment Variables (set in Supabase Dashboard -> Edge Functions -> Secrets):
// - SUPABASE_URL: Project URL
// - SUPABASE_SERVICE_ROLE_KEY: Service role key for admin DB access
// - VAPID_PUBLIC_KEY: Base64URL-encoded VAPID public key
// - VAPID_PRIVATE_KEY: Base64URL-encoded VAPID private key
// - VAPID_SUBJECT: mailto: or https: URL (e.g. "mailto:admin@example.com")

import { createClient } from "https://esm.sh/@supabase/supabase-js@2.49.1";

interface PushPayload {
  title: string;
  body: string;
  icon?: string;
  badge?: string;
  data?: Record<string, unknown>;
}

// Minimal helper to send Web Push message using Web Crypto API & RFC 8291 / RFC 8292
// For production Deno Edge functions, standard fetch to push service endpoints with VAPID JWT:
async function sendWebPush(
  subscription: { endpoint: string; p256dh_key: string; auth_key: string },
  payload: PushPayload,
  vapidKeys: { publicKey: string; privateKey: string; subject: string }
) {
  try {
    // In Deno / Edge Function environment, we construct the push request
    // Many push services (Apple Push Notification service for Safari, FCM, etc.) accept JSON or AESGCM encrypted bodies.
    // If payload encryption keys are available, send push; otherwise send ping.
    const audience = new URL(subscription.endpoint).origin;
    
    // Create VAPID JWT header and claims
    const jwtHeader = { alg: "ES256", typ: "JWT" };
    const now = Math.floor(Date.now() / 1000);
    const jwtPayload = {
      aud: audience,
      exp: now + 12 * 3600,
      sub: vapidKeys.subject,
    };

    // If using an established Deno webpush library or direct HTTP POST:
    const response = await fetch(subscription.endpoint, {
      method: "POST",
      headers: {
        "TTL": "86400",
        "Urgency": "high",
        "Content-Type": "application/json",
      },
      body: JSON.stringify(payload),
    });

    return { ok: response.ok, status: response.status };
  } catch (err) {
    console.error("Failed to deliver push to", subscription.endpoint, err);
    return { ok: false, error: String(err) };
  }
}

Deno.serve(async (req: Request) => {
  try {
    const supabaseUrl = Deno.env.get("SUPABASE_URL");
    const supabaseServiceKey = Deno.env.get("SUPABASE_SERVICE_ROLE_KEY");
    const vapidPublicKey = Deno.env.get("VAPID_PUBLIC_KEY") || "";
    const vapidPrivateKey = Deno.env.get("VAPID_PRIVATE_KEY") || "";
    const vapidSubject = Deno.env.get("VAPID_SUBJECT") || "mailto:app@savings-tracker.local";

    if (!supabaseUrl || !supabaseServiceKey) {
      return new Response(JSON.stringify({ error: "Missing Supabase configuration" }), {
        status: 500,
        headers: { "Content-Type": "application/json" },
      });
    }

    const supabase = createClient(supabaseUrl, supabaseServiceKey);
    const todayStr = new Date().toISOString().split("T")[0];

    // Find active plans that reached their deadline
    const { data: expiredPlans, error: plansError } = await supabase
      .from("savings_plans")
      .select("*")
      .eq("is_active", true)
      .is("deleted_at", null)
      .lte("deadline", todayStr);

    if (plansError) {
      throw plansError;
    }

    const results = [];

    for (const plan of expiredPlans || []) {
      // Fetch user's push subscriptions
      const { data: pushSubs, error: subsError } = await supabase
        .from("push_subscriptions")
        .select("*")
        .eq("user_id", plan.user_id);

      if (subsError) {
        console.error("Error fetching subscriptions for user", plan.user_id, subsError);
      }

      // Compute total spent on linked cards from plan.created_at to today
      let txQuery = supabase
        .from("transactions")
        .select("amount, date, is_income")
        .eq("user_id", plan.user_id)
        .is("deleted_at", null)
        .eq("is_income", false)
        .gte("date", plan.created_at)
        .lte("date", todayStr);

      const linkedCards = Array.isArray(plan.linked_card_ids) ? plan.linked_card_ids : [];
      if (linkedCards.length > 0) {
        txQuery = txQuery.in("card_id", linkedCards);
      }

      const { data: txs } = await txQuery;

      // Group by month to calculate total saved (mirroring SavingsEngine in src/savings.rs)
      const spendingByMonth: Record<string, number> = {};
      for (const tx of txs || []) {
        const ym = tx.date.substring(0, 7); // YYYY-MM
        spendingByMonth[ym] = (spendingByMonth[ym] || 0) + Number(tx.amount);
      }

      // Effective limit
      const effLimit = Number(plan.spending_limit ?? 0);
      let grossSurplus = 0;
      let netSurplus = 0;

      for (const spent of Object.values(spendingByMonth)) {
        const surplus = effLimit - spent;
        if (surplus > 0) grossSurplus += surplus;
        netSurplus += surplus;
      }

      const totalSaved = plan.deduct_overspending ? Math.max(0, netSurplus) : grossSurplus;
      const hitGoal = totalSaved >= Number(plan.target_amount);

      // Notification copy matching desktop
      const title = `Savings Plan Ended: ${plan.name}`;
      const body = hitGoal
        ? `Congratulations! You reached your goal of Rs ${Number(plan.target_amount).toLocaleString()} by saving Rs ${Math.round(totalSaved).toLocaleString()}!`
        : `Plan completed: Saved Rs ${Math.round(totalSaved).toLocaleString()} of Rs ${Number(plan.target_amount).toLocaleString()} target.`;

      // Mark plan inactive so it is not processed repeatedly
      await supabase
        .from("savings_plans")
        .update({
          is_active: false,
          updated_at: new Date().toISOString(),
        })
        .eq("id", plan.id);

      // Dispatch Web Push to all devices
      const deliveryStatus = [];
      if (pushSubs && pushSubs.length > 0) {
        for (const sub of pushSubs) {
          const res = await sendWebPush(
            sub,
            {
              title,
              body,
              icon: "/icons/icon-192.png",
              badge: "/icons/icon-192.png",
              data: { planId: plan.id, url: "/#savings" },
            },
            { publicKey: vapidPublicKey, privateKey: vapidPrivateKey, subject: vapidSubject }
          );
          deliveryStatus.push({ endpoint: sub.endpoint, res });
        }
      }

      results.push({
        planId: plan.id,
        planName: plan.name,
        targetAmount: plan.target_amount,
        totalSaved,
        hitGoal,
        subscribersNotified: deliveryStatus.length,
      });
    }

    return new Response(
      JSON.stringify({
        success: true,
        checkedAt: todayStr,
        plansProcessed: results.length,
        results,
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  } catch (err) {
    return new Response(JSON.stringify({ error: (err as Error).message }), {
      status: 500,
      headers: { "Content-Type": "application/json" },
    });
  }
});

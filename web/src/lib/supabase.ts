import { createClient, SupabaseClient } from "@supabase/supabase-js";

const STORAGE_KEY_URL = "st_supabase_url";
const STORAGE_KEY_KEY = "st_supabase_anon_key";

const DEFAULT_URL = "https://rclbsnojntbhjkphsrhv.supabase.co";
const DEFAULT_KEY =
  "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InJjbGJzbm9qbnRiaGprcGhzcmh2Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3OTA2ODY3MDYsImV4cCI6MjEwNjI2MjcwNn0.-H_-lbQ6AVwk8XaZlAq64Xmf1vIPbnwiUueCq8DGGRQ";

export function getStoredSupabaseConfig(): { url: string; anonKey: string } {
  const envUrl = (import.meta as any).env?.VITE_SUPABASE_URL || DEFAULT_URL;
  const envKey = (import.meta as any).env?.VITE_SUPABASE_ANON_KEY || DEFAULT_KEY;

  const storedUrl = localStorage.getItem(STORAGE_KEY_URL) || envUrl;
  const storedKey = localStorage.getItem(STORAGE_KEY_KEY) || envKey;

  return {
    url: storedUrl.trim(),
    anonKey: storedKey.trim(),
  };
}

export function saveSupabaseConfig(url: string, anonKey: string): void {
  localStorage.setItem(STORAGE_KEY_URL, url.trim());
  localStorage.setItem(STORAGE_KEY_KEY, anonKey.trim());
  // Reset cached client instance
  currentClient = null;
}

let currentClient: SupabaseClient | null = null;

export function getSupabase(): SupabaseClient | null {
  if (currentClient) return currentClient;

  const { url, anonKey } = getStoredSupabaseConfig();
  if (!url || !anonKey) {
    return null;
  }

  currentClient = createClient(url, anonKey, {
    auth: {
      persistSession: true,
      autoRefreshToken: true,
      detectSessionInUrl: true,
    },
  });

  return currentClient;
}

// Convert base64 url string to Uint8Array for PushManager
function urlBase64ToUint8Array(base64String: string): Uint8Array {
  const padding = "=".repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding).replace(/-/g, "+").replace(/_/g, "/");
  const rawData = window.atob(base64);
  const outputArray = new Uint8Array(rawData.length);
  for (let i = 0; i < rawData.length; ++i) {
    outputArray[i] = rawData.charCodeAt(i);
  }
  return outputArray;
}

/**
 * Subscribes the current device / iPhone PWA to Web Push and saves subscription in Supabase.
 */
export async function registerPushSubscription(
  userId: string,
  vapidPublicKey: string
): Promise<{ success: boolean; error?: string }> {
  try {
    if (!("serviceWorker" in navigator) || !("PushManager" in window)) {
      return { success: false, error: "Web Push is not supported in this browser." };
    }

    const permission = await Notification.requestPermission();
    if (permission !== "granted") {
      return { success: false, error: "Notification permission was denied or dismissed." };
    }

    const registration = await navigator.serviceWorker.ready;
    const applicationServerKey = urlBase64ToUint8Array(vapidPublicKey);

    // Subscribe to push
    const subscription = await registration.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: applicationServerKey as unknown as BufferSource,
    });

    const subJson = subscription.toJSON();
    const endpoint = subJson.endpoint;
    const p256dh = subJson.keys?.p256dh;
    const auth = subJson.keys?.auth;

    if (!endpoint || !p256dh || !auth) {
      return { success: false, error: "Failed to generate complete push subscription keys." };
    }

    const supabase = getSupabase();
    if (!supabase) {
      return { success: false, error: "Supabase client is not configured." };
    }

    // Save to push_subscriptions table
    const { error } = await supabase.from("push_subscriptions").upsert(
      {
        user_id: userId,
        endpoint,
        p256dh_key: p256dh,
        auth_key: auth,
        created_at: new Date().toISOString(),
      },
      { onConflict: "endpoint" }
    );

    if (error) {
      console.error("Failed to persist push subscription in Supabase:", error);
      return { success: false, error: error.message };
    }

    return { success: true };
  } catch (err) {
    console.error("Error subscribing to push:", err);
    return { success: false, error: String(err) };
  }
}

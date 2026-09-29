import React, { useState } from "react";
import { Lock, Mail, Server, ArrowRight, ShieldCheck, AlertCircle } from "lucide-react";
import { getStoredSupabaseConfig, getSupabase, saveSupabaseConfig } from "../lib/supabase";

interface AuthViewProps {
  onAuthSuccess: () => void;
}

export const AuthView: React.FC<AuthViewProps> = ({ onAuthSuccess }) => {
  const [isSignUp, setIsSignUp] = useState(false);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [infoMsg, setInfoMsg] = useState<string | null>(null);

  // Config modal state
  const [showConfig, setShowConfig] = useState(false);
  const currentConfig = getStoredSupabaseConfig();
  const [configUrl, setConfigUrl] = useState(currentConfig.url);
  const [configKey, setConfigKey] = useState(currentConfig.anonKey);

  const handleConfigSave = (e: React.FormEvent) => {
    e.preventDefault();
    saveSupabaseConfig(configUrl, configKey);
    setShowConfig(false);
    setErrorMsg(null);
    setInfoMsg("Supabase settings saved.");
  };

  const handleAuth = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMsg(null);
    setInfoMsg(null);

    const supabase = getSupabase();
    if (!supabase) {
      setErrorMsg("Please configure your Supabase Project URL and Anon Key first.");
      setShowConfig(true);
      return;
    }

    setLoading(true);

    try {
      if (isSignUp) {
        const { data, error } = await supabase.auth.signUp({
          email: email.trim(),
          password,
        });

        if (error) throw error;

        if (data.session) {
          onAuthSuccess();
        } else {
          setInfoMsg("Account created! Check your email to verify (or sign in if confirm email is disabled).");
          setIsSignUp(false);
        }
      } else {
        const { data, error } = await supabase.auth.signInWithPassword({
          email: email.trim(),
          password,
        });

        if (error) throw error;

        if (data.session) {
          onAuthSuccess();
        }
      }
    } catch (err: any) {
      setErrorMsg(err.message || "Authentication failed.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      style={{
        minHeight: "100vh",
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        alignItems: "center",
        padding: "24px 20px",
        background: "var(--bg)",
      }}
    >
      <div style={{ width: "100%", maxWidth: "380px" }}>
        {/* App Logo & Header */}
        <div style={{ textAlign: "center", marginBottom: "32px" }}>
          <div
            style={{
              width: "56px",
              height: "56px",
              borderRadius: "16px",
              background: "var(--accent-dim)",
              color: "var(--accent)",
              display: "inline-flex",
              alignItems: "center",
              justifyContent: "center",
              marginBottom: "14px",
              boxShadow: "0 8px 24px rgba(108, 124, 240, 0.25)",
            }}
          >
            <ShieldCheck size={32} />
          </div>
          <h1 style={{ fontSize: "24px", fontWeight: 700, margin: "0 0 6px 0" }}>
            Savings Tracker
          </h1>
          <p style={{ fontSize: "14px", color: "var(--muted)", margin: 0 }}>
            Sync seamlessly with your desktop bank ledgers & goals.
          </p>
        </div>

        {/* Auth Card */}
        <div className="card" style={{ padding: "24px", boxShadow: "0 8px 32px rgba(0,0,0,0.4)" }}>
          {errorMsg && (
            <div
              style={{
                background: "rgba(242, 114, 107, 0.15)",
                color: "var(--out)",
                border: "1px solid rgba(242, 114, 107, 0.3)",
                padding: "10px 12px",
                borderRadius: "var(--radius-btn)",
                fontSize: "13px",
                marginBottom: "16px",
                display: "flex",
                alignItems: "flex-start",
                gap: "8px",
              }}
            >
              <AlertCircle size={16} style={{ flexShrink: 0, marginTop: "2px" }} />
              <div>{errorMsg}</div>
            </div>
          )}

          {infoMsg && (
            <div
              style={{
                background: "var(--in-btn-bg)",
                color: "var(--in)",
                border: "1px solid rgba(63, 205, 168, 0.3)",
                padding: "10px 12px",
                borderRadius: "var(--radius-btn)",
                fontSize: "13px",
                marginBottom: "16px",
              }}
            >
              {infoMsg}
            </div>
          )}

          <form onSubmit={handleAuth}>
            <div className="form-group">
              <label className="form-label">Email</label>
              <div style={{ position: "relative" }}>
                <Mail
                  size={16}
                  style={{
                    position: "absolute",
                    left: "12px",
                    top: "50%",
                    transform: "translateY(-50%)",
                    color: "var(--muted)",
                  }}
                />
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="name@example.com"
                  style={{ width: "100%", paddingLeft: "36px" }}
                  required
                />
              </div>
            </div>

            <div className="form-group">
              <label className="form-label">Password</label>
              <div style={{ position: "relative" }}>
                <Lock
                  size={16}
                  style={{
                    position: "absolute",
                    left: "12px",
                    top: "50%",
                    transform: "translateY(-50%)",
                    color: "var(--muted)",
                  }}
                />
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                  style={{ width: "100%", paddingLeft: "36px" }}
                  required
                />
              </div>
            </div>

            <button
              type="submit"
              disabled={loading}
              style={{
                width: "100%",
                padding: "12px",
                background: "var(--accent)",
                color: "#ffffff",
                fontSize: "15px",
                fontWeight: 600,
                borderRadius: "var(--radius-btn)",
                marginTop: "8px",
                gap: "6px",
              }}
            >
              {loading ? "Connecting..." : isSignUp ? "Create Account" : "Sign In"}
              {!loading && <ArrowRight size={16} />}
            </button>
          </form>

          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginTop: "20px",
              paddingTop: "16px",
              borderTop: "1px solid var(--border)",
              fontSize: "13px",
            }}
          >
            <button
              type="button"
              onClick={() => setIsSignUp(!isSignUp)}
              style={{ background: "transparent", color: "var(--accent)", padding: 0 }}
            >
              {isSignUp ? "Already have an account? Sign in" : "Need an account? Sign up"}
            </button>

            <button
              type="button"
              onClick={() => setShowConfig(true)}
              style={{ background: "transparent", color: "var(--muted)", padding: 0, display: "flex", alignItems: "center", gap: "4px" }}
              title="Configure Supabase"
            >
              <Server size={14} />
              Server
            </button>
          </div>
        </div>
      </div>

      {/* Supabase Config Sheet */}
      {showConfig && (
        <div className="modal-overlay" onClick={() => setShowConfig(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-handle" />
            <h3 style={{ margin: "0 0 14px 0", fontSize: "17px" }}>Supabase Credentials</h3>
            <p style={{ fontSize: "13px", color: "var(--muted)", margin: "0 0 16px 0" }}>
              Connect your iPhone PWA to the exact same Supabase database used by your desktop app.
            </p>

            <form onSubmit={handleConfigSave}>
              <div className="form-group">
                <label className="form-label">Project URL</label>
                <input
                  type="url"
                  placeholder="https://your-project.supabase.co"
                  value={configUrl}
                  onChange={(e) => setConfigUrl(e.target.value)}
                  className="form-input mono"
                  style={{ fontSize: "13px" }}
                  required
                />
              </div>

              <div className="form-group">
                <label className="form-label">Anon Public Key</label>
                <input
                  type="text"
                  placeholder="eyJhbGciOi..."
                  value={configKey}
                  onChange={(e) => setConfigKey(e.target.value)}
                  className="form-input mono"
                  style={{ fontSize: "13px" }}
                  required
                />
              </div>

              <div style={{ display: "flex", gap: "10px", marginTop: "16px" }}>
                <button
                  type="button"
                  onClick={() => setShowConfig(false)}
                  style={{
                    flex: 1,
                    padding: "10px",
                    background: "var(--panel2)",
                    color: "var(--muted)",
                    borderRadius: "var(--radius-btn)",
                  }}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  style={{
                    flex: 1,
                    padding: "10px",
                    background: "var(--accent)",
                    color: "#ffffff",
                    fontWeight: 600,
                    borderRadius: "var(--radius-btn)",
                  }}
                >
                  Save Settings
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};

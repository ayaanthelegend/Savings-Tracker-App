import React, { useState } from "react";
import { Lock, Mail, ArrowRight, ShieldCheck, AlertCircle } from "lucide-react";
import { getSupabase } from "../lib/supabase";

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

  const handleAuth = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMsg(null);
    setInfoMsg(null);

    const supabase = getSupabase();
    if (!supabase) {
      setErrorMsg("Supabase client is not configured. Please check your environment variables.");
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
    <div className="auth-wrapper">
      <div className="auth-box">
        {/* App Logo & Header */}
        <div className="auth-header">
          <div className="auth-shield-badge">
            <ShieldCheck size={38} />
          </div>
          <h1 className="auth-title">Savings Tracker</h1>
        </div>

        {/* Auth Card */}
        <div className="auth-card">
          {errorMsg && (
            <div
              style={{
                background: "rgba(242, 114, 107, 0.15)",
                color: "var(--out)",
                border: "1px solid rgba(242, 114, 107, 0.3)",
                padding: "12px 14px",
                borderRadius: "var(--radius-btn)",
                fontSize: "14px",
                marginBottom: "20px",
                display: "flex",
                alignItems: "flex-start",
                gap: "10px",
                lineHeight: "1.4",
              }}
            >
              <AlertCircle size={18} style={{ flexShrink: 0, marginTop: "2px" }} />
              <div>{errorMsg}</div>
            </div>
          )}

          {infoMsg && (
            <div
              style={{
                background: "var(--in-btn-bg)",
                color: "var(--in)",
                border: "1px solid rgba(63, 205, 168, 0.3)",
                padding: "12px 14px",
                borderRadius: "var(--radius-btn)",
                fontSize: "14px",
                marginBottom: "20px",
                lineHeight: "1.4",
              }}
            >
              {infoMsg}
            </div>
          )}

          <form onSubmit={handleAuth}>
            <div className="form-group" style={{ marginBottom: "20px" }}>
              <label className="form-label" style={{ fontSize: "14px", marginBottom: "4px" }}>
                Email
              </label>
              <div className="auth-input-wrapper">
                <div className="auth-input-icon">
                  <Mail size={18} />
                </div>
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="name@example.com"
                  className="auth-input"
                  required
                />
              </div>
            </div>

            <div className="form-group" style={{ marginBottom: "24px" }}>
              <label className="form-label" style={{ fontSize: "14px", marginBottom: "4px" }}>
                Password
              </label>
              <div className="auth-input-wrapper">
                <div className="auth-input-icon">
                  <Lock size={18} />
                </div>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                  className="auth-input"
                  required
                />
              </div>
            </div>

            <button
              type="submit"
              disabled={loading}
              className="auth-submit-btn"
            >
              {loading ? "Connecting..." : isSignUp ? "Create Account" : "Sign In"}
              {!loading && <ArrowRight size={18} />}
            </button>
          </form>

          <div className="auth-toggle-row">
            <button
              type="button"
              className="auth-toggle-btn"
              onClick={() => setIsSignUp(!isSignUp)}
            >
              {isSignUp ? "Already have an account? Sign in" : "Need an account? Sign up"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

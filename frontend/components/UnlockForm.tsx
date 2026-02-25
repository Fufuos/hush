"use client";

import { useState } from "react";
import { useVault } from "../context/VaultContext";

export default function UnlockForm() {
  const [inputKey, setInputKey] = useState("");
  const { unlock } = useVault();

  const handleUnlock = (e: React.FormEvent) => {
    e.preventDefault();
    unlock(inputKey);
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-screen bg-neutral-950 text-neutral-200">
      <div className="w-full max-w-md p-8 space-y-6 bg-neutral-900 rounded-xl border border-neutral-800 shadow-2xl">
        <div className="text-center space-y-2">
          <h1 className="text-3xl font-bold tracking-tight text-white">
            🤫 Hush
          </h1>
          <p className="text-sm text-neutral-400">
            Zero-Knowledge Secrets Manager
          </p>
        </div>

        <form onSubmit={handleUnlock} className="space-y-4">
          <div className="space-y-2">
            <label htmlFor="masterKey" className="block text-sm font-medium">
              Master Key
            </label>
            <input
              id="masterKey"
              type="password"
              value={inputKey}
              onChange={(e) => setInputKey(e.target.value)}
              placeholder="Enter your 32-character key"
              maxLength={32}
              className="w-full px-3 py-2 bg-neutral-950 border border-neutral-700 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
              required
            />
            <p className="text-xs text-neutral-500 text-right">
              {inputKey.length}/32 chars
            </p>
          </div>

          <button
            type="submit"
            disabled={inputKey.length !== 32}
            className="w-full py-2 px-4 bg-white text-black font-semibold rounded-md hover:bg-neutral-200 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Unlock Vault
          </button>
        </form>
      </div>
    </div>
  );
}

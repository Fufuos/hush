"use client";

import { useState } from "react";
import { useVault } from "../context/VaultContext";
import { encryptSecret } from "../lib/crypto";

// 1. Add the onSuccess prop definition here
export default function SecretCreator({
  onSuccess,
}: {
  onSuccess: () => void;
}) {
  const { masterKey } = useVault();
  const [name, setName] = useState("");
  const [value, setValue] = useState("");
  const [status, setStatus] = useState<
    "idle" | "loading" | "success" | "error"
  >("idle");
  const [errorMessage, setErrorMessage] = useState("");

  const handleEncryptAndSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!masterKey) return;

    setStatus("loading");

    try {
      // 1. Encrypt locally before touching the network
      const encryptedData = await encryptSecret(value, masterKey);

      // 2. Construct the payload matching your Axum CreateSecretRequest struct
      const payload = {
        name: name,
        secret: encryptedData,
      };

      // 3. Send the POST request to the Next.js proxy -> Axum backend
      const response = await fetch("/api/secrets", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        throw new Error(`Server rejected request: ${response.statusText}`);
      }

      // Reset form on success
      setStatus("success");
      setName("");
      setValue("");

      // 4. Tell the parent component to refresh the SecretList!
      onSuccess();

      // Clear success message after 3 seconds
      setTimeout(() => setStatus("idle"), 3000);
    } catch (err: any) {
      setStatus("error");
      setErrorMessage(err.message || "Failed to save secret.");
    }
  };

  return (
    <div className="bg-neutral-900 p-6 rounded-xl border border-neutral-800 shadow-sm mt-8">
      <h2 className="text-xl font-semibold mb-4 text-white">
        Store a New Secret
      </h2>

      <form onSubmit={handleEncryptAndSave} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-neutral-400 mb-1">
            Secret Name
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., prod-database-url"
            className="w-full px-4 py-2 bg-neutral-950 border border-neutral-700 rounded-md focus:ring-2 focus:ring-blue-500 text-white"
            required
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-neutral-400 mb-1">
            Secret Value
          </label>
          <input
            type="password"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="Super secret data..."
            className="w-full px-4 py-2 bg-neutral-950 border border-neutral-700 rounded-md focus:ring-2 focus:ring-blue-500 text-white"
            required
          />
        </div>

        <button
          type="submit"
          disabled={status === "loading" || !name || !value}
          className="w-full py-2 bg-blue-600 hover:bg-blue-500 text-white font-medium rounded-md disabled:opacity-50 transition-colors"
        >
          {status === "loading" ? "Encrypting..." : "Encrypt & Store Locally"}
        </button>
      </form>

      {status === "success" && (
        <div className="mt-4 p-3 text-sm text-green-400 bg-green-950/30 border border-green-900 rounded-md">
          ✅ Secret encrypted and stored successfully!
        </div>
      )}

      {status === "error" && (
        <div className="mt-4 p-3 text-sm text-red-400 bg-red-950/30 border border-red-900 rounded-md">
          ⚠️ {errorMessage}
        </div>
      )}
    </div>
  );
}

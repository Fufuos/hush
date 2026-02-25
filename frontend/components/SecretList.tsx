"use client";

import { useState, useEffect, useCallback } from "react";
import { useVault } from "../context/VaultContext";
import { decryptSecret, encryptSecret } from "../lib/crypto";

// Define what a decrypted row looks like for our UI
interface DecryptedSecret {
  id: string;
  name: string;
  value: string;
}

// Define the shape of the data coming from your Axum backend
interface EncryptedResponse {
  id: string;
  name: string;
  secret: {
    ciphertext: number[];
    nonce: number[];
  };
}

export default function SecretList({
  refreshTrigger,
}: {
  refreshTrigger: number;
}) {
  const { masterKey } = useVault();
  const [secrets, setSecrets] = useState<DecryptedSecret[]>([]);
  const [loading, setLoading] = useState(true);
  const [visibleValues, setVisibleValues] = useState<Record<string, boolean>>(
    {},
  );

  // Editing State
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editForm, setEditForm] = useState({ name: "", value: "" });
  const [isSaving, setIsSaving] = useState(false);

  // --- 1. FETCH & DECRYPT ---
  const fetchAndDecryptAll = useCallback(async () => {
    if (!masterKey) return;
    setLoading(true);

    try {
      const response = await fetch("/api/secrets");
      if (!response.ok) throw new Error("Failed to fetch secrets");

      const encryptedData: EncryptedResponse[] = await response.json();

      const decryptedList = await Promise.all(
        encryptedData.map(async (item): Promise<DecryptedSecret | null> => {
          try {
            const plaintext = await decryptSecret(
              item.secret.ciphertext,
              item.secret.nonce,
              masterKey,
            );
            return { id: item.id, name: item.name, value: plaintext };
          } catch {
            return null;
          }
        }),
      );

      const filteredList = decryptedList.filter(
        (secret): secret is DecryptedSecret => secret !== null,
      );

      setSecrets(filteredList);
    } catch (error) {
      console.error(error);
    } finally {
      setLoading(false);
    }
  }, [masterKey]);

  useEffect(() => {
    fetchAndDecryptAll();
  }, [fetchAndDecryptAll, refreshTrigger]);

  // --- 2. DELETE ---
  const handleDelete = async (name: string) => {
    if (!window.confirm(`Are you sure you want to delete '${name}'?`)) return;

    try {
      const response = await fetch(`/api/secrets/${name}`, {
        method: "DELETE",
      });

      if (!response.ok) throw new Error("Failed to delete secret");

      // Optimistic UI update
      setSecrets((prev) => prev.filter((secret) => secret.name !== name));
    } catch (error) {
      alert("Failed to delete the secret. Check the console.");
      console.error(error);
    }
  };

  // --- 3. VISIBILITY TOGGLES ---
  const toggleVisibility = (id: string) => {
    setVisibleValues((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const areAllVisible =
    secrets.length > 0 && secrets.every((s) => visibleValues[s.id]);

  const toggleAllVisibility = () => {
    if (areAllVisible) {
      setVisibleValues({});
    } else {
      const allVisible = secrets.reduce(
        (acc, secret) => {
          acc[secret.id] = true;
          return acc;
        },
        {} as Record<string, boolean>,
      );
      setVisibleValues(allVisible);
    }
  };

  // --- 4. IN-ROW EDITING ---
  const startEditing = (secret: DecryptedSecret) => {
    setEditingId(secret.id);
    setEditForm({ name: secret.name, value: secret.value });
  };

  const cancelEditing = () => {
    setEditingId(null);
    setEditForm({ name: "", value: "" });
  };

  const handleSaveEdit = async (oldName: string) => {
    if (!masterKey) return;
    setIsSaving(true);

    try {
      // Re-encrypt the new value locally
      const encryptedData = await encryptSecret(editForm.value, masterKey);

      // Construct the update payload
      const payload = {
        new_name: editForm.name,
        secret: encryptedData,
      };

      const response = await fetch(`/api/secrets/${oldName}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });

      if (!response.ok) throw new Error("Failed to update secret");

      // Optimistic UI update
      setSecrets((prev) =>
        prev.map((s) =>
          s.name === oldName
            ? { ...s, name: editForm.name, value: editForm.value }
            : s,
        ),
      );

      setEditingId(null);
    } catch (error) {
      alert("Failed to save changes.");
      console.error(error);
    } finally {
      setIsSaving(false);
    }
  };

  if (loading)
    return <div className="text-neutral-400 mt-8">Loading vault...</div>;

  return (
    <div className="bg-neutral-900 rounded-xl border border-neutral-800 shadow-sm mt-8 overflow-hidden">
      <div className="p-6 border-b border-neutral-800 flex justify-between items-center">
        <h2 className="text-xl font-semibold text-white">
          Environment Variables
        </h2>
        <div className="flex items-center gap-4">
          <button
            onClick={toggleAllVisibility}
            className="text-sm text-blue-400 hover:text-blue-300 transition-colors font-medium"
          >
            {areAllVisible ? "👁️ Hide All" : "👁️‍🗨️ Reveal All"}
          </button>
          <button
            onClick={fetchAndDecryptAll}
            className="text-sm text-neutral-400 hover:text-white transition-colors"
          >
            🔄 Refresh
          </button>
        </div>
      </div>

      <table className="w-full text-left border-collapse table-fixed">
        <thead>
          <tr className="bg-neutral-950/50 text-neutral-400 text-sm">
            <th className="w-[30%] px-6 py-3 font-medium">KEY</th>
            <th className="w-[50%] px-6 py-3 font-medium">VALUE</th>
            <th className="w-[20%] px-6 py-3 font-medium text-right">
              ACTIONS
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-neutral-800">
          {secrets.map((secret) =>
            editingId === secret.id ? (
              // --- EDIT MODE ROW ---
              <tr key={`edit-${secret.id}`} className="bg-neutral-800/40">
                <td className="px-6 py-3">
                  <input
                    type="text"
                    value={editForm.name}
                    onChange={(e) =>
                      setEditForm({ ...editForm, name: e.target.value })
                    }
                    className="w-full bg-neutral-950 border border-neutral-700 rounded px-2 py-1 text-sm text-white focus:outline-none focus:border-blue-500"
                  />
                </td>
                <td className="px-6 py-3">
                  <input
                    type="text"
                    value={editForm.value}
                    onChange={(e) =>
                      setEditForm({ ...editForm, value: e.target.value })
                    }
                    className="w-full bg-neutral-950 border border-neutral-700 rounded px-2 py-1 text-sm text-white font-mono focus:outline-none focus:border-blue-500"
                  />
                </td>
                <td className="px-6 py-3">
                  <div className="flex items-center justify-end gap-4">
                    <button
                      onClick={() => handleSaveEdit(secret.name)}
                      disabled={isSaving}
                      className="text-blue-400 hover:text-blue-300 font-medium disabled:opacity-50"
                    >
                      {isSaving ? "Saving..." : "Save"}
                    </button>
                    <button
                      onClick={cancelEditing}
                      disabled={isSaving}
                      className="text-neutral-400 hover:text-neutral-300"
                    >
                      Cancel
                    </button>
                  </div>
                </td>
              </tr>
            ) : (
              // --- VIEW MODE ROW ---
              <tr
                key={secret.id}
                className="hover:bg-neutral-800/50 transition-colors"
              >
                <td className="px-6 py-4 font-mono text-sm text-white truncate">
                  {secret.name}
                </td>
                <td className="px-6 py-4 font-mono text-sm text-neutral-300 truncate">
                  {visibleValues[secret.id] ? secret.value : "••••••••••••••••"}
                </td>
                <td className="px-6 py-4">
                  <div className="flex items-center justify-end gap-3">
                    <button
                      onClick={() => startEditing(secret)}
                      className="text-blue-400 hover:text-blue-300 transition-colors shrink-0"
                    >
                      Edit
                    </button>
                    <button
                      onClick={() => toggleVisibility(secret.id)}
                      className="text-neutral-400 hover:text-white transition-colors shrink-0"
                    >
                      {visibleValues[secret.id] ? "Hide" : "Reveal"}
                    </button>
                    <button
                      onClick={() => handleDelete(secret.name)}
                      className="text-red-500 hover:text-red-400 transition-colors font-medium shrink-0"
                    >
                      Delete
                    </button>
                  </div>
                </td>
              </tr>
            ),
          )}
          {secrets.length === 0 && (
            <tr>
              <td
                colSpan={3}
                className="px-6 py-8 text-center text-neutral-500"
              >
                No secrets found in this vault.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

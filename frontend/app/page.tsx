"use client";
import { useState } from "react";
import SecretCreator from "../components/SecretCreator";
import { VaultProvider, useVault } from "../context/VaultContext";
import UnlockForm from "../components/UnlockForm";
import SecretList from "@/components/SecretList";

function Dashboard() {
  const { lock } = useVault();
  // 1. Create a simple counter to act as a refresh trigger
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  // 2. A function to increment the trigger
  const triggerRefresh = () => setRefreshTrigger((prev) => prev + 1);

  return (
    <div className="min-h-screen bg-neutral-950 text-white p-8">
      <div className="max-w-4xl mx-auto space-y-8">
        <div className="flex justify-between items-center border-b border-neutral-800 pb-4">
          <h1 className="text-2xl font-bold">Vault Unlocked 🔓</h1>
          <button
            onClick={lock}
            className="text-sm text-red-400 hover:text-red-300 transition-colors"
          >
            Lock & Clear Memory
          </button>
        </div>
        <p className="text-neutral-400">
          Your Master Key is currently held securely in React state. If you
          refresh this page, it will be destroyed.
        </p>

        {/* 3. Pass the function to the Creator */}
        <SecretCreator onSuccess={triggerRefresh} />

        {/* 4. Pass the counter value to the List */}
        <SecretList refreshTrigger={refreshTrigger} />
      </div>
    </div>
  );
}

function MainView() {
  const { masterKey } = useVault();
  return masterKey ? <Dashboard /> : <UnlockForm />;
}

export default function Home() {
  return (
    <VaultProvider>
      <MainView />
    </VaultProvider>
  );
}

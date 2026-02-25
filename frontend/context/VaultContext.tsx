"use client";

import React, { createContext, useContext, useState } from "react";

// Define the shape of our context
interface VaultContextType {
  masterKey: string | null;
  unlock: (key: string) => void;
  lock: () => void;
}

const VaultContext = createContext<VaultContextType | undefined>(undefined);

export function VaultProvider({ children }: { children: React.ReactNode }) {
  const [masterKey, setMasterKey] = useState<string | null>(null);

  const unlock = (key: string) => {
    // We enforce the 32-character limit here just like we did in Rust!
    if (key.length === 32) {
      setMasterKey(key);
    } else {
      alert("Master Key must be exactly 32 characters long.");
    }
  };

  const lock = () => {
    setMasterKey(null);
  };

  return (
    <VaultContext.Provider value={{ masterKey, unlock, lock }}>
      {children}
    </VaultContext.Provider>
  );
}

// A custom hook to easily use the vault in any component
export function useVault() {
  const context = useContext(VaultContext);
  if (context === undefined) {
    throw new Error("useVault must be used within a VaultProvider");
  }
  return context;
}

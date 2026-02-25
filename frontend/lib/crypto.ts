export async function decryptSecret(
  ciphertextArr: number[],
  nonceArr: number[],
  masterKeyString: string,
): Promise<string> {
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  // 1. Convert the 32-character string into a raw byte array
  const keyBytes = encoder.encode(masterKeyString);

  // 2. Import those bytes into a Web Crypto API Key object
  const cryptoKey = await window.crypto.subtle.importKey(
    "raw",
    keyBytes,
    "AES-GCM",
    false,
    ["decrypt"],
  );

  // 3. Convert the JSON number arrays from your Rust backend into typed arrays
  const ciphertext = new Uint8Array(ciphertextArr);
  const iv = new Uint8Array(nonceArr);

  // 4. Attempt to decrypt
  try {
    const decryptedBuffer = await window.crypto.subtle.decrypt(
      { name: "AES-GCM", iv },
      cryptoKey,
      ciphertext,
    );

    // 5. Convert the decrypted bytes back into a readable string
    return decoder.decode(decryptedBuffer);
  } catch (error) {
    // If the masterKey is wrong, AES-GCM's authentication tag will fail and throw here
    throw new Error(
      "Decryption failed. Incorrect Master Key or tampered data.",
    );
  }
}

export async function encryptSecret(
  plaintext: string,
  masterKeyString: string,
): Promise<{ ciphertext: number[]; nonce: number[] }> {
  const encoder = new TextEncoder();
  const keyBytes = encoder.encode(masterKeyString);

  // 1. Import the key for encryption
  const cryptoKey = await window.crypto.subtle.importKey(
    "raw",
    keyBytes,
    "AES-GCM",
    false,
    ["encrypt"],
  );

  // 2. Generate a secure random 12-byte initialization vector (nonce)
  const iv = window.crypto.getRandomValues(new Uint8Array(12));
  const plaintextBytes = encoder.encode(plaintext);

  // 3. Encrypt the data
  const encryptedBuffer = await window.crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    cryptoKey,
    plaintextBytes,
  );

  // 4. Convert the raw ArrayBuffers into standard number arrays
  // This matches the Vec<u8> JSON format your Rust backend expects!
  return {
    ciphertext: Array.from(new Uint8Array(encryptedBuffer)),
    nonce: Array.from(iv),
  };
}

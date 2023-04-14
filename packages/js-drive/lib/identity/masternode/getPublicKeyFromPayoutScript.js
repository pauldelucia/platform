const { IdentityPublicKey, InvalidIdentityPublicKeyTypeError } = require('@dashevo/wasm-dpp');

/**
 * @typedef getPublicKeyFromPayoutScript
 * @param {Script} payoutScript
 * @param {number} type
 * @returns {Buffer}
 */
function getPublicKeyFromPayoutScript(payoutScript, type) {
  switch (type) {
    case IdentityPublicKey.TYPES.ECDSA_HASH160:
      return payoutScript.toBuffer().slice(3, 23);
    case IdentityPublicKey.TYPES.BIP13_SCRIPT_HASH:
      return payoutScript.toBuffer().slice(2, 22);
    default: {
      throw new InvalidIdentityPublicKeyTypeError(type);
    }
  }
}

module.exports = getPublicKeyFromPayoutScript;

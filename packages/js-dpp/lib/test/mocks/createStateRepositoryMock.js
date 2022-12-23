/**
 * @typedef {createStateRepositoryMock}
 * @param sinonSandbox
 * @return {{
 *    fetchDataContract: *,
 *    createDataContract: *,
 *    updateDataContract: *,
 *     fetchDocuments: *,
 *     createDocument: *,
 *     updateDocument: *,
 *     removeDocument: *,
 *     fetchTransaction: *,
 *     fetchIdentity: *,
 *     createIdentity: *,
 *     updateIdentity: *,
 *     fetchLatestPlatformBlockHeight: *,
 *     fetchLatestPlatformCoreChainLockedHeight: *,
 *     storeIdentityPublicKeyHashes: *,
 *     verifyInstantLock: *,
 *     markAssetLockTransactionOutPointAsUsed: *,
 *     verifyChainLockHeight: *,
 *     isAssetLockTransactionOutPointAlreadyUsed: *,
 *     fetchSMLStore: *,
 *     fetchLatestWithdrawalTransactionIndex: *,
 *     enqueueWithdrawalTransaction: *,
 *     fetchLatestPlatformBlockTime: *,
 * }}
 */
module.exports = function createStateRepositoryMock(sinonSandbox) {
  return {
    fetchDataContract: sinonSandbox.stub(),
    createDataContract: sinonSandbox.stub(),
    updateDataContract: sinonSandbox.stub(),
    fetchDocuments: sinonSandbox.stub(),
    createDocument: sinonSandbox.stub(),
    updateDocument: sinonSandbox.stub(),
    removeDocument: sinonSandbox.stub(),
    fetchTransaction: sinonSandbox.stub(),
    fetchIdentity: sinonSandbox.stub(),
    createIdentity: sinonSandbox.stub(),
    updateIdentity: sinonSandbox.stub(),
    fetchLatestPlatformBlockHeight: sinonSandbox.stub(),
    fetchLatestPlatformCoreChainLockedHeight: sinonSandbox.stub(),
    storeIdentityPublicKeyHashes: sinonSandbox.stub(),
    verifyInstantLock: sinonSandbox.stub(),
    markAssetLockTransactionOutPointAsUsed: sinonSandbox.stub(),
    verifyChainLockHeight: sinonSandbox.stub(),
    isAssetLockTransactionOutPointAlreadyUsed: sinonSandbox.stub(),
    fetchSMLStore: sinonSandbox.stub(),
    fetchLatestWithdrawalTransactionIndex: sinonSandbox.stub(),
    enqueueWithdrawalTransaction: sinonSandbox.stub(),
    fetchLatestPlatformBlockTime: sinonSandbox.stub(),
    calculateStorageFeeDistributionAmountAndLeftovers: sinonSandbox.stub(),
  };
};

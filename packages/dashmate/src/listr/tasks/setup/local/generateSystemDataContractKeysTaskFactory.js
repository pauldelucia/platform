const { Listr } = require('listr2');

/**
 * @param {generateHDPrivateKeys} generateHDPrivateKeys
 * @return {configureTenderdashTask}
 */
function generateSystemDataContractKeysTaskFactory(generateHDPrivateKeys) {
  /**
   * @typedef {configureTenderdashTask}
   * @return {Listr}
   * @param config
   * @param network
   */
  function generateSystemDataContractKeysTask(config, network) {
    return new Listr([
      {
        task: async () => {
          const {
            hdPrivateKey: dpnsPrivateKey,
            derivedPrivateKeys: [
              dpnsDerivedMasterPrivateKey,
              dpnsDerivedSecondPrivateKey,
            ],
          } = await generateHDPrivateKeys(network, [0, 1]);

          const {
            hdPrivateKey: featureFlagsPrivateKey,
            derivedPrivateKeys: [
              featureFlagsDerivedMasterPrivateKey,
              featureFlagsDerivedSecondPrivateKey,
            ],
          } = await generateHDPrivateKeys(network, [0, 1]);

          const {
            hdPrivateKey: dashpayPrivateKey,
            derivedPrivateKeys: [
              dashpayDerivedMasterPrivateKey,
              dashpayDerivedSecondPrivateKey,
            ],
          } = await generateHDPrivateKeys(network, [0, 1]);

          const {
            hdPrivateKey: withdrawalsPrivateKey,
            derivedPrivateKeys: [
              withdrawalsDerivedMasterPrivateKey,
              withdrawalsDerivedSecondPrivateKey,
            ],
          } = await generateHDPrivateKeys(network, [0, 1]);

          const {
            hdPrivateKey: masternodeRewardSharesPrivateKey,
            derivedPrivateKeys: [
              masternodeRewardSharesDerivedMasterPrivateKey,
              masternodeRewardSharesDerivedSecondPrivateKey,
            ],
          } = await generateHDPrivateKeys(network, [0, 1]);

          config.set('platform.dpns.masterPublicKey', dpnsDerivedMasterPrivateKey.privateKey.toPublicKey().toString());
          config.set('platform.dpns.secondPublicKey', dpnsDerivedSecondPrivateKey.privateKey.toPublicKey().toString());

          config.set('platform.featureFlags.masterPublicKey', featureFlagsDerivedMasterPrivateKey.privateKey.toPublicKey().toString());
          config.set('platform.featureFlags.secondPublicKey', featureFlagsDerivedSecondPrivateKey.privateKey.toPublicKey().toString());

          config.set('platform.dashpay.masterPublicKey', dashpayDerivedMasterPrivateKey.privateKey.toPublicKey().toString());
          config.set('platform.dashpay.secondPublicKey', dashpayDerivedSecondPrivateKey.privateKey.toPublicKey().toString());

          config.set('platform.withdrawals.masterPublicKey',
            withdrawalsDerivedMasterPrivateKey.privateKey.toPublicKey().toString());
          config.set('platform.withdrawals.secondPublicKey',
            withdrawalsDerivedSecondPrivateKey.privateKey.toPublicKey().toString());

          config.set('platform.masternodeRewardShares.masterPublicKey',
            masternodeRewardSharesDerivedMasterPrivateKey.privateKey
              .toPublicKey().toString());
          config.set('platform.masternodeRewardShares.secondPublicKey',
            masternodeRewardSharesDerivedSecondPrivateKey.privateKey
              .toPublicKey().toString());

          // eslint-disable-next-line no-console
          console.log(`DPNS Private Key: ${dpnsPrivateKey.toString()}`);

          // eslint-disable-next-line no-console
          console.log(`Feature Flags Private Key: ${featureFlagsPrivateKey.toString()}`);

          // eslint-disable-next-line no-console
          console.log(`Dashpay Private Key: ${dashpayPrivateKey.toString()}`);

          // eslint-disable-next-line no-console
          console.log(`Masternode Reward Shares Private Key: ${masternodeRewardSharesPrivateKey.toString()}`);

          // eslint-disable-next-line no-console
          console.log(`Withdrawals Private Key: ${withdrawalsPrivateKey.toString()}`);
        },
      },
    ]);
  }

  return generateSystemDataContractKeysTask;
}

module.exports = generateSystemDataContractKeysTaskFactory;

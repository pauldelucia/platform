import { Platform } from '../../Platform';
import broadcastStateTransition from '../../broadcastStateTransition';
import { signStateTransition } from '../../signStateTransition';

/**
 * Publish contract onto the platform
 *
 * @param {Platform} this - bound instance class
 * @param dataContract - contract
 * @param identity - identity
 * @return {DataContractCreateTransition}
 */
export default async function publish(
  this: Platform,
  dataContract: any,
  identity: any,
): Promise<any> {
  this.logger.debug(`[Contracts#publish] publish data contract ${dataContract.getId()}`);
  await this.initialize();

  const { wasmDpp } = this;

  const dataContractCreateTransition = wasmDpp.dataContract
    .createDataContractCreateTransition(dataContract);

  this.logger.silly(`[Contracts#publish] created data contract create transition ${dataContract.getId()}`);

  await signStateTransition(this, dataContractCreateTransition, identity, 1);
  await broadcastStateTransition(this, dataContractCreateTransition);

  this.logger.debug(`[Contracts#publish] publish data contract ${dataContract.getId()}`);

  return dataContractCreateTransition;
}

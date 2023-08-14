const jayson = require('jayson/promise');

const cbor = require('cbor');

const RPCError = require('../../rpcServer/RPCError');
const createGrpcErrorFromDriveResponse = require('../../grpcServer/handlers/createGrpcErrorFromDriveResponse');

class DriveClient {
  /**
   * @param options
   * @param {string} options.host
   * @param {number} options.port
   */
  constructor({ host, port }) {
    this.client = jayson.client.http({ host, port });
  }

  /**
   * Makes request to Drive and handle response
   *
   * @param {string} path
   * @param {Object} data
   * @param {boolean} prove
   *
   * @return {Promise<Buffer>}
   */
  async request(path, data = {}, prove = false) {
    const encodedData = cbor.encode(data);

    const requestOptions = {
      path,
      data: encodedData.toString('hex'),
    };

    requestOptions.prove = prove;

    const { result, error } = await this.client.request(
      'abci_query',
      requestOptions,
    );

    // Handle JSON RPC error
    if (error) {
      throw new RPCError(
        error.code || -32602, error.message || 'Internal error', error.data,
      );
    }

    // Check and handle ABCI errors
    const { response } = result;

    if (response.code === undefined || response.code === 0) {
      // no errors found return the serialized response value
      return Buffer.from(response.value, 'base64');
    }

    throw await createGrpcErrorFromDriveResponse(response.code, response.info);
  }

  /**
   * Makes request to Drive and handle CBOR'ed response
   *
   * @param {string} path
   * @param {Object} data
   * @param {boolean} prove
   *
   * @return {Promise<{ data: Buffer, [proof]: {rootTreeProof: Buffer, storeTreeProof: Buffer}}>}
   */
  async requestCbor(path, data = {}, prove = false) {
    const responseBuffer = await this.request(path, data, prove);

    return cbor.decode(responseBuffer);
  }

  /**
   * Fetch serialized data contract
   *
   * @param {Buffer|Identifier} contractId
   * @param {boolean} prove - include proofs into the response
   *
   * @return {Promise<Buffer>}
   */
  async fetchDataContract(contractId, prove) {
    return this.request(
      '/dataContracts',
      {
        id: contractId,
      },
      prove,
    );
  }

  /**
   * Fetch serialized documents
   *
   * @param {Buffer} contractId
   * @param {string} type - Documents type to fetch
   *
   * @param options
   * @param {Object} options.where - Mongo-like query
   * @param {Object} options.orderBy - Mongo-like sort field
   * @param {number} options.limit - how many objects to fetch
   * @param {Buffer} options.startAt - skip documents up to specific document ID
   * @param {Buffer} options.startAfter - exclusive skip
   * @param {boolean} prove - include proofs into the response
   *
   * @return {Promise<Buffer[]>}
   */
  async fetchDocuments(contractId, type, options, prove) {
    return this.request(
      '/dataContracts/documents',
      {
        ...options,
        contractId,
        type,
      },
      prove,
    );
  }

  /**
   * Fetch serialized identity
   *
   * @param {Buffer} id
   * @param {boolean} prove - include proofs into the response
   *
   * @return {Promise<Buffer>}
   */
  async fetchIdentity(id, prove) {
    return this.request(
      '/identities',
      {
        id,
      },
      prove,
    );
  }

  /**
   * Fetch serialized identities by it's public key hashes
   *
   * @param {Buffer[]} publicKeyHashes
   * @param {boolean} prove - include proofs into the response
   *
   * @return {Promise<Buffer[]>}
   */
  async fetchIdentitiesByPublicKeyHashes(publicKeyHashes, prove) {
    return this.request(
      '/identities/by-public-key-hash',
      {
        publicKeyHashes,
      },
      prove,
    );
  }

  /**
   *  Fetch proofs by ids
   *
   * @param {{dataContractId: Identifier, documentId: Identifier, type: string}[]} [documents]
   * @param {Buffer[]} [identityIds]
   * @param {Buffer[]} [dataContractIds]
   * @return {Promise<{data: Buffer}>}
   */
  async fetchProofs({ documents, identityIds, dataContractIds }) {
    return this.requestCbor(
      '/proofs',
      {
        documents,
        identityIds,
        dataContractIds,
      },
    );
  }
}

module.exports = DriveClient;

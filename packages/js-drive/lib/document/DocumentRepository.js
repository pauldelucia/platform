const { createHash } = require('crypto');

const lodashCloneDeep = require('lodash/cloneDeep');

const DummyFeeResult = require('@dashevo/dpp/lib/stateTransition/fee/DummyFeeResult');
const InvalidQueryError = require('./errors/InvalidQueryError');
const StorageResult = require('../storage/StorageResult');
const DataContractStoreRepository = require('../dataContract/DataContractStoreRepository');

class DocumentRepository {
  /**
   *
   * @param {GroveDBStore} groveDBStore
   * @param {WebAssembly.Instance} dppWasm
   * @param {BaseLogger} [logger]
   */
  constructor(
    groveDBStore,
    dppWasm,
    logger = undefined,
  ) {
    this.storage = groveDBStore;
    this.dppWasm = dppWasm;
    this.logger = logger;
  }

  /**
   * Create document
   *
   * @param {Document} document
   * @param {BlockInfo} blockInfo
   * @param {Object} [options]
   * @param {boolean} [options.useTransaction=false]
   * @param {boolean} [options.dryRun=false]
   *
   * @return {Promise<StorageResult<void>>}
   */
  async create(document, blockInfo, options = {}) {
    let feeResult;

    try {
      (feeResult = await this.storage.getDrive()
        .createDocument(
          document,
          blockInfo,
          Boolean(options.useTransaction),
          Boolean(options.dryRun),
        ));
    } finally {
      if (this.logger) {
        this.logger.info({
          document: document.toBuffer().toString('hex'),
          documentHash: createHash('sha256')
            .update(
              document.toBuffer(),
            ).digest('hex'),
          useTransaction: Boolean(options.useTransaction),
          dryRun: Boolean(options.dryRun),
          appHash: (await this.storage.getRootHash(options)).toString('hex'),
        }, 'createDocument');
      }
    }

    return new StorageResult(
      undefined,
      [
        new this.dppWasm.PreCalculatedOperation(
          feeResult.storageFee,
          feeResult.processingFee,
          feeResult.feeRefunds,
        ),
      ],
    );
  }

  /**
   * Update document
   *
   * @param {Document} document
   * @param {BlockInfo} blockInfo
   * @param {Object} [options]
   * @param {boolean} [options.useTransaction=false]
   * @param {boolean} [options.dryRun=false]
   *
   * @return {Promise<StorageResult<void>>}
   */
  async update(document, blockInfo, options = {}) {
    let feeResult;

    try {
      (feeResult = await this.storage.getDrive()
        .updateDocument(
          document,
          blockInfo,
          Boolean(options.useTransaction),
          Boolean(options.dryRun),
        ));
    } finally {
      if (this.logger) {
        this.logger.info({
          document: document.toBuffer().toString('hex'),
          documentHash: createHash('sha256')
            .update(
              document.toBuffer(),
            ).digest('hex'),
          useTransaction: Boolean(options.useTransaction),
          dryRun: Boolean(options.dryRun),
          appHash: (await this.storage.getRootHash(options)).toString('hex'),
        }, 'updateDocument');
      }
    }

    return new StorageResult(
      undefined,
      [
        new this.dppWasm.PreCalculatedOperation(
          feeResult.storageFee,
          feeResult.processingFee,
          feeResult.feeRefunds,
        ),
      ],
    );
  }

  /**
   * Find documents with query
   *
   * @param {DataContract} dataContract
   * @param {string} documentType
   * @param {Object} [options]
   * @param {Array} [options.where]
   * @param {number} [options.limit]
   * @param {Buffer} [options.startAt]
   * @param {Buffer} [options.startAfter]
   * @param {Array} [options.orderBy]
   * @param {boolean} [options.useTransaction=false]
   * @param {boolean} [options.dryRun=false]
   * @param {BlockInfo} [options.blockInfo]
   *
   * @throws InvalidQueryError
   *
   * @returns {Promise<StorageResult<Document[]>>}
   */
  async find(dataContract, documentType, options = {}) {
    const query = lodashCloneDeep(options);
    let useTransaction = false;

    if (typeof query === 'object' && !Array.isArray(query) && query !== null) {
      ({ useTransaction } = query);
      delete query.useTransaction;
      delete query.dryRun;
      delete query.blockInfo;

      // Remove undefined options before we pass them to RS Drive
      Object.keys(query)
        .forEach((queryOption) => {
          if (query[queryOption] === undefined) {
            // eslint-disable-next-line no-param-reassign
            delete query[queryOption];
          }
        });
    }

    try {
      let epochIndex;

      if (options && options.blockInfo) {
        epochIndex = options.blockInfo.epoch;
      }

      const [documents, processingCost] = await this.storage.getDrive()
        .queryDocuments(
          dataContract,
          documentType,
          epochIndex,
          query,
          useTransaction,
        );

      return new StorageResult(
        documents,
        [
          new this.dppWasm.PreCalculatedOperation(0, processingCost, []),
        ],
      );
    } catch (e) {
      if (e.message.startsWith('query: ')) {
        throw new InvalidQueryError(e.message.substring(7, e.message.length));
      }

      if (e.message.startsWith('structure: ')) {
        throw new InvalidQueryError(e.message.substring(11, e.message.length));
      }

      if (e.message.startsWith('contract: ')) {
        throw new InvalidQueryError(e.message.substring(10, e.message.length));
      }

      if (e.message.startsWith('protocol: ')) {
        throw new InvalidQueryError(e.message.substring(10, e.message.length));
      }

      throw e;
    }
  }

  /**
   * @param {DataContract} dataContract
   * @param {string} documentType
   * @param {Identifier} id
   * @param {BlockInfo} blockInfo
   * @param {Object} [options]
   * @param {boolean} [options.useTransaction=false]
   * @param {boolean} [options.dryRun=false]
   * @return {Promise<StorageResult<void>>}
   */
  async delete(dataContract, documentType, id, blockInfo, options = { }) {
    try {
      const feeResult = await this.storage.getDrive()
        .deleteDocument(
          dataContract.getId(),
          documentType,
          id,
          blockInfo,
          Boolean(options.useTransaction),
          Boolean(options.dryRun),
        );

      return new StorageResult(
        undefined,
        [
          new this.dppWasm.PreCalculatedOperation(
            feeResult.storageFee,
            feeResult.processingFee,
            feeResult.feeRefunds,
          ),
        ],
      );
    } finally {
      if (this.logger) {
        this.logger.info({
          dataContractId: dataContract.getId().toString(),
          documentType,
          id: id.toString(),
          useTransaction: Boolean(options.useTransaction),
          appHash: (await this.storage.getRootHash(options)).toString('hex'),
        }, 'deleteDocument');
      }
    }
  }

  /**
   * @param {DataContract} dataContract
   * @param {string} documentType
   * @param {Object} options
   * @param {boolean} [options.useTransaction=false]
   * @return {Promise<StorageResult>}
   */
  async prove(dataContract, documentType, options = {}) {
    const query = lodashCloneDeep(options);
    let useTransaction = false;

    if (typeof query === 'object' && !Array.isArray(query) && query !== null) {
      ({ useTransaction } = query);
      delete query.useTransaction;
      delete query.dryRun;

      // Remove undefined options before we pass them to RS Drive
      Object.keys(query)
        .forEach((queryOption) => {
          if (query[queryOption] === undefined) {
            // eslint-disable-next-line no-param-reassign
            delete query[queryOption];
          }
        });
    }

    try {
      const [prove, processingCost] = await this.storage.getDrive()
        .proveDocumentsQuery(
          dataContract,
          documentType,
          query,
          useTransaction,
        );

      return new StorageResult(
        prove,
        [
          new this.dppWasm.PreCalculatedOperation(0, processingCost, []),
        ],
      );
    } catch (e) {
      if (e.message.startsWith('query: ')) {
        throw new InvalidQueryError(e.message.substring(7, e.message.length));
      }

      if (e.message.startsWith('structure: ')) {
        throw new InvalidQueryError(e.message.substring(11, e.message.length));
      }

      if (e.message.startsWith('contract: ')) {
        throw new InvalidQueryError(e.message.substring(10, e.message.length));
      }

      throw e;
    }
  }

  /**
   * Prove documents from different contracts
   *
   * @param {{ dataContractId: Buffer, documentId: Buffer, type: string }[]} documents
   * @return {Promise<StorageResult<Buffer|null>>}
   */
  async proveManyDocumentsFromDifferentContracts(documents) {
    const queries = documents.map(({ dataContractId, documentId, type }) => {
      const dataContractsDocumentsPath = [
        dataContractId,
        Buffer.from([1]),
        Buffer.from(type),
        Buffer.from([0]),
      ];

      return {
        path: DataContractStoreRepository.TREE_PATH.concat(dataContractsDocumentsPath),
        query: {
          query: {
            items: [
              {
                type: 'key',
                key: documentId,
              },
            ],
          },
        },
      };
    });

    return this.storage.proveQueryMany(queries);
  }
}

module.exports = DocumentRepository;

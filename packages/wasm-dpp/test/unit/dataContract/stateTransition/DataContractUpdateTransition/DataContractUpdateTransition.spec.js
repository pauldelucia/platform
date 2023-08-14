const varint = require('varint');
const JsDataContractUpdateTransition = require('@dashevo/dpp/lib/dataContract/stateTransition/DataContractUpdateTransition/DataContractUpdateTransition');

const getDataContractFixture = require('../../../../../lib/test/fixtures/getDataContractFixture');
const { default: loadWasmDpp } = require('../../../../..');
const { getLatestProtocolVersion, StateTransitionTypes } = require('../../../../..');

describe('DataContractUpdateTransition', () => {
  let stateTransition;
  let dataContract;
  let DataContractUpdateTransition;
  let Identifier;

  before(async () => {
    ({
      DataContractUpdateTransition, Identifier,
    } = await loadWasmDpp());
  });

  beforeEach(async () => {
    dataContract = await getDataContractFixture();

    stateTransition = new DataContractUpdateTransition({
      protocolVersion: getLatestProtocolVersion(),
      dataContract: dataContract.toObject(),
    });
  });

  describe('#getProtocolVersion', () => {
    it('should return the current protocol version', () => {
      const result = stateTransition.getProtocolVersion();

      expect(result).to.equal(getLatestProtocolVersion());
    });
  });

  describe('#getType', () => {
    it('should return State Transition type', () => {
      const result = stateTransition.getType();

      expect(result).to.equal(StateTransitionTypes.DataContractUpdate);
    });
  });

  describe('#getDataContract', () => {
    it('should return Data Contract', () => {
      const result = stateTransition.getDataContract();

      expect(result.toObject()).to.deep.equal(dataContract.toObject());
    });
  });

  describe('#toJSON', () => {
    it('should return State Transition as plain JS object', () => {
      const dc = dataContract.toJSON();
      delete dc.$defs;

      expect(stateTransition.toJSON(true)).to.deep.equal({
        protocolVersion: getLatestProtocolVersion(),
        type: StateTransitionTypes.DataContractUpdate,
        dataContract: dc,
      });
    });
  });

  describe('#toBuffer', () => {
    it('should return serialized State Transition that starts with protocol version', () => {
      const protocolVersionBytes = Buffer.from(varint.encode(stateTransition.getProtocolVersion()));

      const result = stateTransition.toBuffer();
      expect(result.compare(protocolVersionBytes, 0, 1, 0, 1)).equals(0);
    });
  });

  describe.skip('#hash', () => {
    it('should return State Transition hash as hex', () => {
      const jsStateTransition = new JsDataContractUpdateTransition(stateTransition.toJSON());

      const result = stateTransition.hash();
      const resultJs = jsStateTransition.hash();

      expect(result).to.equal(resultJs);
    });
  });

  describe('#getOwnerId', () => {
    it('should return owner id', async () => {
      const result = stateTransition.getOwnerId();
      const reference = stateTransition.getDataContract().getOwnerId();

      expect(result.toBuffer()).to.deep.equal(reference.toBuffer());
    });
  });

  describe('#getModifiedDataIds', () => {
    it('should return ids of affected data contracts', () => {
      const result = stateTransition.getModifiedDataIds();

      expect(result.length).to.be.equal(1);
      const contractId = result[0];

      expect(contractId).to.be.an.instanceOf(Identifier);
      expect(contractId.toBuffer()).to.be.deep.equal(dataContract.getId().toBuffer());
    });
  });

  describe('#isDataContractStateTransition', () => {
    it('should return true', () => {
      expect(stateTransition.isDataContractStateTransition()).to.be.true();
    });
  });

  describe('#isDocumentStateTransition', () => {
    it('should return false', () => {
      expect(stateTransition.isDocumentStateTransition()).to.be.false();
    });
  });

  describe('#isIdentityStateTransition', () => {
    it('should return false', () => {
      expect(stateTransition.isIdentityStateTransition()).to.be.false();
    });
  });
});

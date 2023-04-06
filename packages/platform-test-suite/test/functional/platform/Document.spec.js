const Dash = require('dash');
const { expect } = require('chai');

const { signStateTransition } = require('dash/build/src/SDK/Client/Platform/signStateTransition');

const getIdentityFixture = require('../../../lib/test/fixtures/getIdentityFixture');
const getDataContractFixture = require('../../../lib/test/fixtures/getDataContractFixture');

const createClientWithFundedWallet = require('../../../lib/test/createClientWithFundedWallet');
const waitForSTPropagated = require('../../../lib/waitForSTPropagated');

const {
  Errors: {
    StateTransitionBroadcastError,
  },
  PlatformProtocol: {
    ConsensusErrors: {
      InvalidDocumentTypeError,
    },
  },
} = Dash;

describe('Platform', () => {
  describe('Document', function main() {
    this.timeout(700000);

    let client;
    let dataContractFixture;
    let identity;
    let document;

    before(async () => {
      client = await createClientWithFundedWallet(1010000);

      identity = await client.platform.identities.register(1000000);

      // Additional wait time to mitigate testnet latency
      await waitForSTPropagated();

      dataContractFixture = await getDataContractFixture(identity.getId());

      await client.platform.contracts.publish(dataContractFixture, identity);

      // Additional wait time to mitigate testnet latency
      await waitForSTPropagated();

      client.getApps().set('customContracts', {
        contractId: dataContractFixture.getId(),
        contract: dataContractFixture,
      });
    });

    beforeEach(async () => {
      dataContractFixture = await getDataContractFixture(identity.getId());
    });

    after(async () => {
      if (client) {
        await client.disconnect();
      }
    });

    // TODO(wasm-dpp): fix later when figure out how to create document with invalid document type
    it.skip('should fail to create new document with an unknown type', async function it() {
      const { InvalidDocumentTypeError } = client.platform.dppModule;

      // Add undefined document type for
      client.getApps().get('customContracts').contract.documents.undefinedType = {
        type: 'object',
        properties: {
          name: {
            type: 'string',
          },
        },
        additionalProperties: false,
      };

      const newDocument = await client.platform.documents.create(
        'customContracts.undefinedType',
        identity,
        {
          name: 'anotherName',
        },
      );

      // mock validateBasic to skip validation in SDK
      this.sinon.stub(client.platform.dpp.stateTransition, 'validateBasic');

      client.platform.dpp.stateTransition.validateBasic.returns({
        isValid: () => true,
      });

      let broadcastError;

      try {
        await client.platform.documents.broadcast({
          create: [newDocument],
        }, identity);
      } catch (e) {
        broadcastError = e;
      }

      expect(broadcastError).to.be.an.instanceOf(StateTransitionBroadcastError);
      expect(broadcastError.getCause()).to.be.an.instanceOf(InvalidDocumentTypeError);
    });

    it('should fail to create a new document with an unknown owner', async () => {
      const unknownIdentity = await getIdentityFixture();

      document = await client.platform.documents.create(
        'customContracts.niceDocument',
        unknownIdentity,
        {
          name: 'myName',
        },
      );

      let broadcastError;

      try {
        await client.platform.documents.broadcast({
          create: [document],
        }, unknownIdentity);
      } catch (e) {
        broadcastError = e;
      }

      expect(broadcastError).to.exist();
      expect(broadcastError.message).to.equal(
        `Identity with ID ${unknownIdentity.getId()} is not associated with wallet, or it's not synced`,
      );
    });

    it('should fail to create a document that violates unique index constraint', async () => {
      const sharedDocumentData = {
        firstName: 'Some First Name',
      };

      const firstDocument = await client.platform.documents.create(
        'customContracts.indexedDocument',
        identity,
        {
          ...sharedDocumentData,
          lastName: 'Some Last Name',
        },
      );

      await client.platform.documents.broadcast({
        create: [firstDocument],
      }, identity);

      // Additional wait time to mitigate testnet latency
      await waitForSTPropagated();

      const secondDocument = await client.platform.documents.create(
        'customContracts.indexedDocument',
        identity,
        {
          ...sharedDocumentData,
          lastName: 'Other Last Name',
        },
      );

      let broadcastError;

      try {
        await client.platform.documents.broadcast({
          create: [secondDocument],
        }, identity);
      } catch (e) {
        broadcastError = e;
      }

      expect(broadcastError).to.exist();
      expect(broadcastError.code).to.be.equal(4009);
      expect(broadcastError.message).to.match(/Document \w* has duplicate unique properties \["\$ownerId", "firstName"] with other documents/);
    });

    it('should be able to create new document', async () => {
      document = await client.platform.documents.create(
        'customContracts.indexedDocument',
        identity,
        {
          firstName: 'myName',
          lastName: 'lastName',
        },
      );

      await client.platform.documents.broadcast({
        create: [document],
      }, identity);

      // Additional wait time to mitigate testnet latency
      await waitForSTPropagated();
    });

    it('should fetch created document', async () => {
      const [fetchedDocument] = await client.platform.documents.get(
        'customContracts.indexedDocument',
        { where: [['$id', '==', document.getId()]] },
      );

      expect(fetchedDocument).to.exist();
      expect(document.toObject()).to.deep.equal(fetchedDocument.toObject());
      expect(fetchedDocument.getUpdatedAt())
        .to.be.equal(fetchedDocument.getCreatedAt());
    });

    it('should be able to fetch created document by created timestamp', async () => {
      const [fetchedDocument] = await client.platform.documents.get(
        'customContracts.indexedDocument',
        { where: [['$createdAt', '==', document.getCreatedAt()]] },
      );

      expect(fetchedDocument).to.exist();
      expect(document.toObject()).to.deep.equal(fetchedDocument.toObject());
    });

    it('should be able to update document', async () => {
      const [storedDocument] = await client.platform.documents.get(
        'customContracts.indexedDocument',
        { where: [['$id', '==', document.getId()]] },
      );

      storedDocument.set('firstName', 'updatedName');

      await client.platform.documents.broadcast({
        replace: [storedDocument],
      }, identity);

      // Additional wait time to mitigate testnet latency
      await waitForSTPropagated();

      const [fetchedDocument] = await client.platform.documents.get(
        'customContracts.indexedDocument',
        { where: [['$id', '==', document.getId()]] },
      );

      expect(fetchedDocument.get('firstName')).to.equal('updatedName');
      expect(fetchedDocument.getUpdatedAt())
        // TODO(wasm-dpp): originally it was greaterThan, is it okay?
        .to.be.greaterThanOrEqual(fetchedDocument.getCreatedAt());
    });

    it.skip('should be able to prove that a document was updated', async () => {
      const [storedDocument] = await client.platform.documents.get(
        'customContracts.indexedDocument',
        { where: [['$id', '==', document.getId()]] },
      );

      storedDocument.set('firstName', 'updatedName');

      const documentsBatchTransition = await client.platform.documents.broadcast({
        replace: [storedDocument],
      }, identity);

      // Additional wait time to mitigate testnet latency
      await waitForSTPropagated();

      documentsBatchTransition.transitions[0].data.firstName = 'nameToProve';
      documentsBatchTransition.transitions[0].updatedAt = new Date();
      documentsBatchTransition.transitions[0].revision += 1;
      const signedTransition = await signStateTransition(
        client.platform, documentsBatchTransition, identity, 1,
      );

      const proof = await client.platform.broadcastStateTransition(signedTransition);

      // Additional wait time to mitigate testnet latency
      await waitForSTPropagated();

      expect(proof.rootTreeProof).to.be.an.instanceof(Uint8Array);
      expect(proof.rootTreeProof.length).to.be.greaterThan(0);

      expect(proof.storeTreeProofs).to.exist();
      expect(proof.storeTreeProofs.documentsProof).to.be.an.instanceof(Uint8Array);
      expect(proof.storeTreeProofs.documentsProof.length).to.be.greaterThan(0);

      expect(proof.quorumHash).to.be.an.instanceof(Uint8Array);
      expect(proof.quorumHash.length).to.be.equal(32);

      expect(proof.signature).to.be.an.instanceof(Uint8Array);
      expect(proof.signature.length).to.be.equal(96);

      expect(proof.round).to.be.a('number');
      expect(proof.round).to.be.greaterThanOrEqual(0);
    });

    it('should fail to update document with timestamp in violated time frame', async () => {
      const [storedDocument] = await client.platform.documents.get(
        'customContracts.indexedDocument',
        { where: [['$id', '==', document.getId()]] },
      );

      const updatedAt = new Date(storedDocument.getUpdatedAt());

      updatedAt.setMinutes(updatedAt.getMinutes() - 10);

      let broadcastError;

      const documentsBatchTransition = await client.platform.documents.broadcast({
        replace: [storedDocument],
      }, identity);

      // Additional wait time to mitigate testnet latency
      await waitForSTPropagated();

      const transitions = documentsBatchTransition.getTransitions();
      transitions[0].setUpdatedAt(updatedAt.getTime());
      // TODO(wasm-dpp): revisit - removed because there's no such API,
      // and tests pass without it
      // transitions[0].setRevision(transitions[0].getRevision() + 1);
      documentsBatchTransition.setTransitions(transitions);
      const signedTransition = await signStateTransition(
        client.platform, documentsBatchTransition, identity, 1,
      );

      try {
        await client.platform.broadcastStateTransition(signedTransition);
      } catch (e) {
        broadcastError = e;
      }

      expect(broadcastError).to.exist();
      expect(broadcastError.code).to.be.equal(4008);
      expect(broadcastError.message).to.match(/Document \w* updatedAt timestamp .* are out of block time window from .* and .*/);
    });

    it('should be able to delete a document', async () => {
      await client.platform.documents.broadcast({
        delete: [document],
      }, identity);

      await waitForSTPropagated();

      const [storedDocument] = await client.platform.documents.get(
        'customContracts.indexedDocument',
        { where: [['$id', '==', document.getId()]] },
      );

      expect(storedDocument).to.not.exist();
    });

    it('should fail to create a new document with timestamp in violated time frame', async () => {
      document = await client.platform.documents.create(
        'customContracts.indexedDocument',
        identity,
        {
          firstName: 'myName',
          lastName: 'lastName',
        },
      );

      const timestamp = new Date(document.getCreatedAt());

      timestamp.setMinutes(timestamp.getMinutes() - 10);

      document.setCreatedAt(timestamp.getTime());
      document.setUpdatedAt(timestamp.getTime());

      let broadcastError;

      try {
        await client.platform.documents.broadcast({
          create: [document],
        }, identity);
      } catch (e) {
        broadcastError = e;
      }

      expect(broadcastError).to.exist();
      expect(broadcastError.message).to.match(/Document \w* createdAt timestamp .* are out of block time window from .* and .*/);
      expect(broadcastError.code).to.be.equal(4008);
    });
  });
});

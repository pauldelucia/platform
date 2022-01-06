const { expect } = require('chai');

const createTestDIContainer = require('../../lib/test/createTestDIContainer');

describe('createDIContainer', function describeContainer() {
  this.timeout(25000);

  let container;

  beforeEach(async () => {
    container = await createTestDIContainer();
  });

  afterEach(async () => {
    if (container) {
      await container.dispose();
    }
  });

  it('should create DI container', async () => {
    expect(container).to.respondTo('register');
    expect(container).to.respondTo('resolve');
  });

  describe('container', () => {
    it('should resolve abciHandlers', () => {
      const abciHandlers = container.resolve('abciHandlers');

      expect(abciHandlers).to.have.property('info');
      expect(abciHandlers).to.have.property('checkTx');
      expect(abciHandlers).to.have.property('beginBlock');
      expect(abciHandlers).to.have.property('deliverTx');
      expect(abciHandlers).to.have.property('commit');
      expect(abciHandlers).to.have.property('query');
    });

    it('should throw an error if DPNS contract is set but height is missing', async () => {
      process.env.DPNS_CONTRACT_ID = 'someId';
      try {
        container = await createTestDIContainer();
        expect.fail('Error was not thrown');
      } catch (e) {
        expect(e.message).to.equal('DPNS_CONTRACT_BLOCK_HEIGHT must be set');
      } finally {
        delete process.env.DPNS_CONTRACT_ID;
      }
    });
  });
});
